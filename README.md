# 2048 game in rust

- using https://lib.rs/crates/pixels
- winit is a bit of a pain... but it means x-platform without an effort,
    hopefully

## Build

For whatever platform you're on:

```sh
cargo build 
```

### cross-compile for windows 

Ensure the target `x86_64-pc-windows-gnu` is added with rustup. (`rustup target
list` and `rustup target add`)

```sh
cargo build --release --target x86_64-pc-windows-gnu
```

Then, zip up the .dll and .exe files from `target/x86_64-pc-windows-gnu/release/` and send them off!

```sh
./release_windows.sh
```

## Coalesce Algorithm

for each item, from end to start
  shift right until you find a wall, or an item, then try to merge
    can merge if 1) same values, 2) hasn't been merged already this coalesce

## Todo

- render numbers
- end of game
- score
- draw game as inset within window
- gutter colors
- windows build
