# Onze

A football tactics board written in Rust with [egui](https://github.com/emilk/egui). One codebase runs natively on Linux, macOS and Windows, and in the browser via WebAssembly.

**Try it in the browser: https://hulkhongan300.github.io/Onze/**

- Pick one of 20 formations (4-3-3, 4-2-3-1, 4-4-2, 3-5-2, 4-1-4-1, 4-4-1-1, 3-4-3, 5-3-2, 4-1-2-1-2 Diamond, 3-4-2-1, 4-5-1, 5-4-1, 4-3-2-1 Christmas Tree, 4-2-2-2 Box, 4-3-1-2, 3-4-1-2, 4-2-4, and the historic 2-3-5 Pyramid, 3-2-2-3 WM and 1-4-3-2 Catenaccio); named ones show their nickname in the dropdown
- Half field (default) or full field; white or grass pitch
- Drag players; they snap to a 9×13 grid, with a dashed line back to their formation spot
- Move tool: box-select or click players, then drag them as a group
- Freehand pen with 5 quick colours plus a custom one, and an eraser, drawn on top of the pitch
- Right-click the pitch for a quick menu (tool, colour, undo, clear)
- Type the team in the side panel: each player has an editable shirt number and name, kept when the formation changes
- Import a real team (Liga Portugal, Premier League, La Liga): pick league and team, and its most recent starting XI fills the board with names and numbers in the formation it played, each player in their position (see [Team data](#team-data))
- Label players by number, name, both (name under the token) or nothing; press `H` to hide the side panel
- Undo (Ctrl/Cmd+Z), clear drawings, reset players
- Save positions: moved players become their new spots (no dashed lines) and are kept, with names, numbers, formation and pitch settings, across refreshes and restarts (drawings are not)

## Team data

Team lineups come from [API-Football](https://www.api-football.com/) but the app never calls it: the API key would be visible to anyone in a public web page or binary. Instead the `teams` GitHub workflow runs [`scripts/fetch_teams.py`](scripts/fetch_teams.py) every Tuesday, writes each team's most recent starting XI and formation to [`data/teams.json`](data/teams.json), commits it if it changed, and redeploys the web version. The file is compiled into the app, so it works offline. Until the workflow has run, the app shows "No teams yet".

One-time setup:

1. Get a free key at [dashboard.api-football.com](https://dashboard.api-football.com/).
2. In the repo, go to *Settings → Secrets and variables → Actions* and add the secret `API_FOOTBALL_KEY`.
3. Under *Actions → teams*, click *Run workflow*.

A run uses about 35 requests, well inside the free plan's 100 per day. **The free plan only serves older seasons** (currently 2022–2024), so with a free key the lineups are from the newest season it allows, and the app says which season in the Import button's tooltip. A paid plan gets the current season automatically. To pin a season, add the repository variable `API_FOOTBALL_SEASON` (for example `2024`). Desktop downloads include the data of the release they were built from.

Run it locally: `API_FOOTBALL_KEY=... python3 scripts/fetch_teams.py`.

## Download

| System | Download |
|---|---|
| Windows (64-bit) | [**Download Onze for Windows** (.exe)](https://github.com/hulkhongan300/Onze/releases/latest/download/onze-windows-x86_64.exe) |
| macOS (Apple Silicon and Intel) | [**Download Onze for macOS** (.dmg)](https://github.com/hulkhongan300/Onze/releases/latest/download/onze-macos.dmg) |
| Linux (x86_64) | [**Download Onze for Linux**](https://github.com/hulkhongan300/Onze/releases/latest/download/onze-linux-x86_64) |

- **Windows:** run the `.exe`.
- **macOS:** open the `.dmg` and drag Onze to Applications.
- **Linux:** make the file executable once (`chmod +x onze-linux-x86_64`), then run it.

All versions are on the [Releases page](https://github.com/hulkhongan300/Onze/releases).

The apps are not code-signed, so the first launch needs one extra step:
- **Windows:** SmartScreen may warn; click *More info* → *Run anyway*.
- **macOS:** right-click Onze → *Open* (or `xattr -dr com.apple.quarantine /Applications/Onze.app`).

To publish a new release, bump the version in `Cargo.toml` and push a tag: `git tag v0.2.0 && git push origin v0.2.0`.

## Run from source

Native:

```sh
cargo run --release
```

Web (needs `rustup target add wasm32-unknown-unknown` and `cargo install --locked trunk`):

```sh
trunk serve          # http://127.0.0.1:8080
trunk build --release  # static site in dist/
```

Tests: `cargo test` and `python3 -m unittest discover -s scripts`

## License

Licensed under the [Apache License 2.0](LICENSE-APACHE).
