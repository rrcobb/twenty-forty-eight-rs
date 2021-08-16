# 2048 game in rust

- using https://lib.rs/crates/pixels
- winit is a bit of a pain... but it means x-platform without an effort,
    hopefully

## Build

For whatever platform you're on:

```sh
cargo build 
```

For some other platform

```sh
cargo build --target [platform]
```

and `--release` to build in release mode.

```sh
cargo build --release --target x86_64-pc-windows-gnu
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
