# Onze

A football tactics board written in Rust with [egui](https://github.com/emilk/egui). One codebase runs natively on Linux, macOS and Windows, and in the browser via WebAssembly.

**Try it in the browser: https://hulkhongan300.github.io/Onze/**

- Pick one of 10 common formations (4-3-3, 4-2-3-1, 4-4-2, 3-5-2, 4-1-4-1, 4-4-1-1, 3-4-3, 5-3-2, 4-1-2-1-2, 3-4-2-1)
- Half field (default) or full field; white or grass pitch
- Drag players; they snap to a 9×13 grid, with a dashed line back to their formation spot
- Move tool: box-select or click players, then drag them as a group
- Freehand pen with 5 quick colours plus a custom one, and an eraser, drawn on top of the pitch
- Right-click the pitch for a quick menu (tool, colour, undo, clear)
- Hide player numbers for a minimal view; press `H` to hide the side panel
- Undo (Ctrl/Cmd+Z), clear drawings, reset players
- Save positions: moved players become their new spots (no dashed lines) for this session; a refresh or restart goes back to defaults

## Run

Native:

```sh
cargo run --release
```

Web (needs `rustup target add wasm32-unknown-unknown` and `cargo install --locked trunk`):

```sh
trunk serve          # http://127.0.0.1:8080
trunk build --release  # static site in dist/
```

Tests: `cargo test`

## License

Licensed under the [Apache License 2.0](LICENSE-APACHE).
