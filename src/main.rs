use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

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
}

impl World {
    /// Create a new `World` instance with empty values
    fn new() -> Self {
        let mut empty = Self {
            values: [[None; 4]; 4]
        };
        empty.add_random_block();
        empty
    }

    fn add_random_block(&mut self) {
        // put a Some where one of the None's are
        self.values[2][1] = Some(2);
    }

    /// Update the `World` internal state; coalesce and add a new box
    fn update(&mut self, input: &WinitInputHelper) {
        if input.key_pressed(VirtualKeyCode::Down) {
            for i in 0..self.values.len() {
                self.values[i] = coalesce(self.values[i]);
            }
        }
        // need to reverse before and after?
        if input.key_pressed(VirtualKeyCode::Up) {
            for i in 0..self.values.len() {
                let mut values = self.values[i];
                values.reverse();
                values = coalesce(values);
                values.reverse();
                self.values[i] = values;
            }
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
        }
        self.add_random_block();
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        let cell_width = (WIDTH / 4) as i16;

        // for each of the cells in values
        // check whether this pixel is in that cell
        // TODO: flip to iterate through values instead of through pixels
        // TODO: Add gutter between items
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let rgba = match self.values[(x / cell_width) as usize][(y / cell_width) as usize] {
                None => [0x00, 0x00, 0x00, 0xff],
                Some(2) => [0xee, 0xe4, 0xda, 0xff],
                Some(4) => [0xee, 0xe1, 0xc9, 0xff],
                Some(8) => [0xf3, 0xb2, 0x7a, 0xff],
                Some(16) => [0xf6, 0x96, 0x64, 0xff],
                Some(32) => [0xf7, 0x7c, 0x5f, 0xff],
                Some(64) => [0xf7, 0x5f, 0x3b, 0xff], // f75f3b
                Some(128) => [0xed, 0xd0, 0x73, 0xff],
                Some(256) => [0xed, 0xcc, 0x62, 0xff],
                Some(512) => [0xed, 0xc9, 0x50, 0xff], // edc950
                Some(1024) => [0xed, 0xc5, 0x3f, 0xff], // edc53f
                Some(2048) => [0xed, 0xc2, 0x2e, 0xff], // edc22e
                Some(_) => [0x48, 0xb2, 0xe8, 0xff], 
            };

            pixel.copy_from_slice(&rgba);
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
        coalesce(&mut arr);
        assert_eq!(arr, [None, None, None, None]);
    }

    #[test]
    fn coalesce_one() {
        let mut arr = [None, None, None, Some(1)];
        coalesce(&mut arr);
        assert_eq!(arr, [None, None, None, Some(1)]);
    }

    #[test]
    fn coalesce_right() {
        let mut arr = [None, None, Some(1), None];
        coalesce(&mut arr);
        assert_eq!(arr, [None, None, None, Some(1)]);
    }

    #[test]
    fn coalesce_add() {
        let mut arr = [None, None, Some(1), Some(1)];
        coalesce(&mut arr);
        assert_eq!(arr, [None, None, None, Some(2)]);
    }

    #[test]
    fn coalesce_add_extra() {
        let mut arr = [None, Some(1), Some(1), Some(1)];
        coalesce(&mut arr);
        assert_eq!(arr, [None, None, Some(1), Some(2)]);
    }

    #[test]
    fn coalesce_all_ones() {
        let mut arr = [Some(1), Some(1), Some(1), Some(1)];
        coalesce(&mut arr);
        assert_eq!(arr, [None, None, Some(2), Some(2)]);
    }

    #[test]
    fn coalesce_leave_two_after_ones() {
        let mut arr = [None, Some(2), Some(1), Some(1)];
        coalesce(&mut arr);
        assert_eq!(arr, [None, None, Some(2), Some(2)]);
    }
}
