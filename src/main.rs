use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use rand::{rngs::ThreadRng, thread_rng, Rng};

const WIDTH: u32 = 720;
const HEIGHT: u32 = 720;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("2048")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| println!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update(&input);
            window.request_redraw();
        }
    });
}

/// Representation of the application state
/// 4x4 grid of values
struct World {
    values: [[Option<u32>; 4]; 4],
    rng: ThreadRng,
}

impl World {
    /// Create a new `World` instance with empty values
    fn new() -> Self {
        let mut empty = Self {
            values: [[None; 4]; 4],
            rng: thread_rng(),
        };
        empty.add_random_block();
        empty
    }

    fn add_random_block(&mut self) {
        let mut nones = vec![];
        for i in 0..4 {
            for j in 0..4 {
                if self.values[i][j] == None { nones.push((i,j)); }
            }
        }
        let random_index = self.rng.gen_range(0..nones.len());
        let (i,j) = nones[random_index];
        println!("adding a random block at {},{}", i, j);
        self.values[i][j] = Some(2);
    }

    /// Update the `World` internal state; coalesce and add a new box
    fn update(&mut self, input: &WinitInputHelper) {
        let mut changed = false;
        if input.key_pressed(VirtualKeyCode::Down) {
            for i in 0..self.values.len() {
                self.values[i] = coalesce(self.values[i]);
            }
            changed = true;
        }
        if input.key_pressed(VirtualKeyCode::Up) {
            for i in 0..self.values.len() {
                let mut values = self.values[i];
                values.reverse();
                values = coalesce(values);
                values.reverse();
                self.values[i] = values;
            }
            changed = true;
        }
        if input.key_pressed(VirtualKeyCode::Right) {
            for i in 0..self.values[0].len() {
                let mut values = [
                    self.values[0][i],
                    self.values[1][i],
                    self.values[2][i],
                    self.values[3][i],
                ];
                values = coalesce(values);
                self.values[0][i] = values[0];
                self.values[1][i] = values[1];
                self.values[2][i] = values[2];
                self.values[3][i] = values[3];
            }
            changed = true;
        }
        if input.key_pressed(VirtualKeyCode::Left) {
            for i in 0..self.values[0].len() {
                let mut values = [
                    self.values[3][i],
                    self.values[2][i],
                    self.values[1][i],
                    self.values[0][i],
                ];
                values = coalesce(values);
                self.values[3][i] = values[0];
                self.values[2][i] = values[1];
                self.values[1][i] = values[2];
                self.values[0][i] = values[3];
            }
            changed = true;
        }
        if changed { self.add_random_block(); }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        let cell_width = (WIDTH / 4) as usize;

        for (col, cells) in self.values.iter().enumerate() {
            for (row, cell) in cells.iter().enumerate() {
                // color based on the value of the cell
                let rgba: [u8; 4] = match cell {
                    None => [0x00, 0x00, 0x00, 0xff],
                    Some(2) => [0xff, 0xf4, 0xea, 0xff],
                    Some(4) => [0xee, 0xe1, 0xc9, 0xff],
                    Some(8) => [0xf3, 0xb2, 0x7a, 0xff],
                    Some(16) => [0xf6, 0x96, 0x64, 0xff],
                    Some(32) => [0xf7, 0x7c, 0x5f, 0xff],
                    Some(64) => [0xf7, 0x5f, 0x3b, 0xff],
                    Some(128) => [0xed, 0xd0, 0x73, 0xff],
                    Some(256) => [0xed, 0xcc, 0x62, 0xff],
                    Some(512) => [0xed, 0xc9, 0x50, 0xff],
                    Some(1024) => [0xed, 0xc5, 0x3f, 0xff],
                    Some(2048) => [0xed, 0xc2, 0x2e, 0xff],
                    Some(_) => [0x48, 0xb2, 0xe8, 0xff], 
                };

                // where to paint in the frame?
                // The frame has all the pixels, in rows WIDTH * 4 wide
                // So, the slices to paint to are at 
                // row * WIDTH * 4 + col * cell_width * 4, and are cell_width * 4 in length
                // and we want to do it cell_height times (square, so it's the cell_width)

                let pixels_to_paint = std::iter::repeat(rgba).take(cell_width).flatten().collect::<Vec<_>>();
                for i in 0..cell_width {
                    let target_start = ((row * cell_width + i) * WIDTH as usize * 4) + (col as usize * cell_width * 4);
                    frame[target_start..(target_start+cell_width*4)]
                        .copy_from_slice(&pixels_to_paint);
                }
            }
        }
    }
}

fn coalesce(mut cells: [Option<u32>; 4]) -> [Option<u32>; 4] {
    let mut merged = [false; 4];
    for index in 0..4 {
        let i = 3 - index;
        let item = cells[i];
        let mut next_i = i + 1;
        if let Some(val) = item {
            // while not out of bounds and the next cell is empty
            while let Some(None) = cells.get(next_i) {
                // move right
                cells[next_i - 1] = None;
                cells[next_i] = item;
                next_i += 1;
            }
            // if there is an opportunity to merge right, do so
            if let Some(Some(_)) = cells.get(next_i) {
                // check that we haven't merged that spot yet
                if !merged[next_i] && Some(val) == cells[next_i] {
                    cells[next_i - 1] = None;
                    cells[next_i] = Some(val + val);
                    merged[next_i] = true;
                }
            }
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use crate::coalesce;
    #[test]
    fn coalesce_nones() {
        let mut arr = [None, None, None, None];
        arr = coalesce(arr);
        assert_eq!(arr, [None, None, None, None]);
    }

    #[test]
    fn coalesce_one() {
        let mut arr = [None, None, None, Some(1)];
        arr = coalesce(arr);
        assert_eq!(arr, [None, None, None, Some(1)]);
    }

    #[test]
    fn coalesce_right() {
        let mut arr = [None, None, Some(1), None];
        arr = coalesce(arr);
        assert_eq!(arr, [None, None, None, Some(1)]);
    }

    #[test]
    fn coalesce_add() {
        let mut arr = [None, None, Some(1), Some(1)];
        arr = coalesce(arr);
        assert_eq!(arr, [None, None, None, Some(2)]);
    }

    #[test]
    fn coalesce_add_extra() {
        let mut arr = [None, Some(1), Some(1), Some(1)];
        arr = coalesce(arr);
        assert_eq!(arr, [None, None, Some(1), Some(2)]);
    }

    #[test]
    fn coalesce_all_ones() {
        let mut arr = [Some(1), Some(1), Some(1), Some(1)];
        arr = coalesce(arr);
        assert_eq!(arr, [None, None, Some(2), Some(2)]);
    }

    #[test]
    fn coalesce_leave_two_after_ones() {
        let mut arr = [None, Some(2), Some(1), Some(1)];
        arr = coalesce(arr);
        assert_eq!(arr, [None, None, Some(2), Some(2)]);
    }
}
