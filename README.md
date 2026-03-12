# Ignis — Modular Arcade Cabinet

A modular, open-source arcade cabinet environment built for the desktop. Named after the Latin word for *fire/glow*, Ignis delivers retro gaming through a clean, plugin-driven architecture powered by WebAssembly.

```
┌─────────────────────────────────────────────────┐
│              TypeScript Display Layer           │
│   PixiJS Renderer · Shell UI · Input Listeners  │
└───────────────────┬─────────────────────────────┘
                    │ Tauri invoke() / emit()
┌───────────────────▼─────────────────────────────┐
│              Rust Host Engine                   │
│   60Hz Game Loop · Input Abstraction · SQLite   │
└───────────────────┬─────────────────────────────┘
                    │ IGI (WIT ABI)
┌───────────────────▼─────────────────────────────┐
│           WASM Plugin Runtime (wasmtime)        │
│   Sandboxed game modules · Plugin discovery     │
└─────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Runtime | [Tauri v2](https://v2.tauri.app/) (Rust backend + webview frontend) |
| Host Engine | Rust 1.94 — game loop, state, persistence |
| Plugin Runtime | [Wasmtime 42](https://wasmtime.dev/) — WASM Component Model |
| Plugin Interface | [WIT](https://component-model.bytecodealliance.org/design/wit.html) + wit-bindgen 0.40 |
| Rendering | [PixiJS v8](https://pixijs.com/) — GPU-accelerated 2D via WebGL/WebGPU |
| Frontend | TypeScript · React 19 · React Router · Vite 8 |
| Persistence | SQLite via sqlx (players, scores, keybindings, plugin storage) |
| Platforms | Windows · macOS · Linux |

## Project Structure

```
ignis/
├── src-tauri/                  # Rust backend (Tauri)
│   ├── src/
│   │   ├── lib.rs              # App entry, Tauri commands, state
│   │   ├── engine/             # GameEngine, 60Hz loop, shared types
│   │   ├── plugin/             # Wasmtime runtime, plugin discovery
│   │   ├── input/              # Input dispatch (frontend → WASM)
│   │   └── db/                 # SQLite init, player CRUD
│   ├── wit/ignis-game.wit      # IGI contract (WIT definition)
│   └── migrations/             # SQL schema migrations
├── src/                        # TypeScript frontend
│   ├── pages/                  # MainMenu, GameView, PlayerSelect, Settings
│   ├── renderer/               # PixiJS app, FrameRenderer
│   ├── input/                  # Keyboard capture, action mapping
│   ├── lib/                    # Tauri wrappers, types, render bridge
│   └── components/             # Layout shell
├── plugins/                    # Game plugins
│   └── hello-world/            # Test plugin (draws rect + text)
│       ├── ignis.toml          # Plugin manifest
│       ├── hello_world.wasm    # Compiled WASM component (~64KB)
│       └── plugin/             # Rust source for the plugin
└── docs/                       # Private — spec, plan, checklist (not in repo)
```

## Ignis Game Interface (IGI)

Games are WASM Component Model modules that implement the IGI contract defined in WIT:

```wit
package ignis:game@0.1.0;

interface host-api {
    draw-sprite: func(id: u32, x: f32, y: f32);
    draw-rect:   func(x: f32, y: f32, w: f32, h: f32, color: u32);
    draw-text:   func(text: string, x: f32, y: f32, size: u8);
    play-sound:  func(id: u32);
    get-storage: func(key: string) -> option<string>;
    set-storage: func(key: string, value: string);
}

world ignis-game {
    import host-api;
    export init:         func();
    export update:       func(delta-ms: u32);
    export handle-input: func(action: u32);
    export get-name:     func() -> string;
    export get-version:  func() -> string;
    export get-author:   func() -> string;
}
```

Plugins are fully sandboxed — no filesystem, network, or OS access. They communicate with the host exclusively through the IGI imports.

## Current Status

**Phase I — The Shell** is complete. The app has:

- Tauri v2 desktop window (960×720)
- Rust game engine with 60 Hz tick loop
- Wasmtime-based WASM Component Model plugin loading
- Plugin discovery via `ignis.toml` manifests
- PixiJS rendering pipeline (`RenderFrame` events at 60fps)
- Keyboard input capture with canonical action mapping (8 actions)
- Full input dispatch: keyboard → TypeScript → Rust → WASM guest
- Player profile CRUD (SQLite-backed)
- Main menu with game selector (keyboard nav + click)
- Hello-world test plugin (red rectangle + "Hello Ignis!" text)
- ESC to return to menu from any game

### Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| I | The Shell — engine, plugins, rendering, input, menu | **Done** |
| II | Standard Wave — hardened host imports, Space Invaders, Tetris, high scores, keybindings, gamepad | Planned |
| III | Complex Wave — Asteroids, Bomberman, plugin browser | Planned |
| IV | Advanced Wave — Pac-Man (A* + ghost FSM), Downwell clone (procgen + gravity) | Planned |
| V | Polish — audio mixing, pixel font, cover art, CI/CD, documentation | Planned |

## Prerequisites

- [Rust](https://rustup.rs/) (1.94.0+) with `wasm32-wasip2` target — wasmtime 42 requires a recent stable toolchain
- [Node.js](https://nodejs.org/) (v22+)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

```bash
# Install WASM target for plugin compilation
rustup target add wasm32-wasip2
```

## Quick Start

```bash
# Clone
git clone https://github.com/alvarotolentino/ignis.git
cd ignis

# Install frontend dependencies
npm install

# Run in development mode
npx tauri dev
```

The app opens a 960×720 window. Select **Hello World** from the menu and press Enter to launch the test plugin.

## Building a Plugin

Plugins are Rust libraries compiled to `wasm32-wasip2`. Minimal example:

```rust
// Cargo.toml: [lib] crate-type = ["cdylib"]
// Dependencies: wit-bindgen = "0.40"

wit_bindgen::generate!({
    path: "wit",
    world: "ignis-game",
    exports: { world: MyGame },
});

struct MyGame;

impl Guest for MyGame {
    fn init() { /* setup */ }

    fn update(delta_ms: u32) {
        use ignis::game::host_api;
        host_api::draw_rect(10.0, 10.0, 50.0, 50.0, 0xFF0000FF);
        host_api::draw_text("Hello!", 10.0, 80.0, 16);
    }

    fn handle_input(action: u32) { /* 0=Up,1=Down,2=Left,3=Right,4=A,5=B,6=Start,7=Select */ }
    fn get_name() -> String { "My Game".into() }
    fn get_version() -> String { "0.1.0".into() }
    fn get_author() -> String { "You".into() }
}
```

```bash
cargo build --target wasm32-wasip2 --release
```

Place the `.wasm` binary and an `ignis.toml` manifest in `plugins/your-game/`:

```toml
[game]
name = "My Game"
version = "0.1.0"
author = "You"

[display]
description = "A short description"
```

## Controls

| Key | Action |
|-----|--------|
| Arrow keys / WASD | Directional movement |
| Z / Space | Action A (confirm / fire) |
| X | Action B (cancel / alt) |
| Enter | Start |
| Shift | Select |
| ESC | Return to menu |

## Development

```bash
# Run dev server
npx tauri dev

# Rust tests
cd src-tauri && cargo test

# Clippy lints
cd src-tauri && cargo clippy --all-targets -- -D warnings

# TypeScript type check
npx tsc --noEmit

# Build plugin
cd plugins/hello-world/plugin
cargo build --target wasm32-wasip2 --release
```

## License

MIT

## Author

**Alvaro Tolentino** — [GitHub](https://github.com/alvarotolentino)
