pub mod discovery;

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::engine::types::RenderCommand;
use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

// Generate typed bindings from the IGI WIT definition.
wasmtime::component::bindgen!({
    path: "wit",
    world: "ignis-game",
});

/// Host state stored in the wasmtime `Store`.
/// Provides WASI context, resource table, a per-frame draw command buffer,
/// and a synchronous SQLite connection for plugin storage.
struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
    /// Accumulated render commands from the current tick's host import calls.
    frame_buffer: Vec<RenderCommand>,
    /// Plugin ID used to scope storage operations.
    plugin_id: String,
    /// Synchronous SQLite connection for get_storage / set_storage.
    /// Wrapped in Arc<Mutex<>> so PluginRuntime is Send + Sync.
    storage_db: Arc<Mutex<rusqlite::Connection>>,
    /// Set by `set_storage("last_score", ...)` — signals game over to the engine.
    game_over_score: Option<u32>,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl ignis::game::host_api::Host for HostState {
    fn draw_sprite(&mut self, id: u32, x: f32, y: f32) {
        self.frame_buffer.push(RenderCommand::DrawSprite { id, x, y });
    }

    fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        if w < 0.0 || h < 0.0 {
            log::warn!("draw_rect: negative dimensions w={w}, h={h} — skipping");
            return;
        }
        if !x.is_finite() || !y.is_finite() || !w.is_finite() || !h.is_finite() {
            log::warn!("draw_rect: non-finite coordinates — skipping");
            return;
        }
        self.frame_buffer
            .push(RenderCommand::DrawRect { x, y, w, h, color });
    }

    fn draw_text(&mut self, text: String, x: f32, y: f32, size: u8) {
        if size == 0 {
            log::warn!("draw_text: size is 0 — skipping");
            return;
        }
        if text.len() > 255 {
            log::warn!(
                "draw_text: text length {} exceeds 255 — truncating",
                text.len()
            );
            let truncated: String = text.chars().take(255).collect();
            self.frame_buffer
                .push(RenderCommand::DrawText { text: truncated, x, y, size });
            return;
        }
        if !x.is_finite() || !y.is_finite() {
            log::warn!("draw_text: non-finite coordinates — skipping");
            return;
        }
        self.frame_buffer
            .push(RenderCommand::DrawText { text, x, y, size });
    }

    fn play_sound(&mut self, id: u32) {
        self.frame_buffer.push(RenderCommand::PlaySound { id });
    }

    fn get_storage(&mut self, key: String) -> Option<String> {
        let db = self.storage_db.lock().expect("storage_db lock poisoned");
        let result = db.query_row(
            "SELECT value FROM plugin_storage WHERE plugin_id = ?1 AND key = ?2",
            rusqlite::params![&self.plugin_id, &key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(val) => Some(val),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => {
                log::warn!("get_storage({key:?}) failed: {e}");
                None
            }
        }
    }

    fn set_storage(&mut self, key: String, value: String) {
        // Detect game-over signal: plugin writes "last_score" on death
        if key == "last_score" {
            if let Ok(score) = value.parse::<u32>() {
                self.game_over_score = Some(score);
            }
        }

        let db = self.storage_db.lock().expect("storage_db lock poisoned");
        let result = db.execute(
            "INSERT INTO plugin_storage (plugin_id, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(plugin_id, key) DO UPDATE SET value = excluded.value",
            rusqlite::params![&self.plugin_id, &key, &value],
        );
        if let Err(e) = result {
            log::warn!("set_storage({key:?}) failed: {e}");
        }
    }
}

/// Runtime wrapper around a loaded WASM component plugin.
/// Encapsulates the wasmtime Store and generated bindings.
pub struct PluginRuntime {
    store: Store<HostState>,
    bindings: IgnisGame,
}

impl PluginRuntime {
    /// Loads a WASM component from disk, links WASI and IGI host imports,
    /// and instantiates the component.
    ///
    /// # Arguments
    /// * `wasm_path` — Path to the `.wasm` component file.
    /// * `plugin_id` — Unique ID for scoping storage (typically the directory name).
    /// * `db_path` — Path to the SQLite database file for plugin storage.
    ///
    /// # Errors
    /// Returns a wasmtime error if the file cannot be read, the component
    /// is invalid, or imports cannot be satisfied.
    pub fn load(wasm_path: &Path, plugin_id: &str, db_path: &Path) -> wasmtime::Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;

        let component = Component::from_file(&engine, wasm_path)?;

        let mut linker: Linker<HostState> = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync::<HostState>(&mut linker)?;
        IgnisGame::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

        // Open synchronous SQLite connection for plugin storage
        let storage_db = rusqlite::Connection::open(db_path)
            .map_err(|e| wasmtime::Error::msg(format!("Failed to open storage DB: {e}")))?;

        // Ensure the plugin_storage table exists
        storage_db
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS plugin_storage (
                    plugin_id TEXT NOT NULL,
                    key TEXT NOT NULL,
                    value TEXT NOT NULL,
                    PRIMARY KEY (plugin_id, key)
                );",
            )
            .map_err(|e| wasmtime::Error::msg(format!("Failed to create storage table: {e}")))?;

        let wasi = WasiCtxBuilder::new().build();
        let mut store = Store::new(
            &engine,
            HostState {
                wasi,
                table: ResourceTable::new(),
                frame_buffer: Vec::new(),
                plugin_id: plugin_id.to_string(),
                storage_db: Arc::new(Mutex::new(storage_db)),
                game_over_score: None,
            },
        );

        let bindings = IgnisGame::instantiate(&mut store, &component, &linker)?;

        log::info!("WASM plugin loaded from {}", wasm_path.display());
        Ok(Self { store, bindings })
    }

    /// Calls the plugin's `init()` export.
    pub fn call_init(&mut self) -> wasmtime::Result<()> {
        self.bindings.call_init(&mut self.store)
    }

    /// Calls the plugin's `update(delta_ms)` export.
    pub fn call_update(&mut self, delta_ms: u32) -> wasmtime::Result<()> {
        self.bindings.call_update(&mut self.store, delta_ms)
    }

    /// Calls the plugin's `handle_input(action)` export.
    pub fn call_handle_input(&mut self, action: u32) -> wasmtime::Result<()> {
        self.bindings.call_handle_input(&mut self.store, action)
    }

    /// Calls the plugin's `get_name()` export.
    #[allow(dead_code)] // Used by plugin discovery UI in later phases
    pub fn call_get_name(&mut self) -> wasmtime::Result<String> {
        self.bindings.call_get_name(&mut self.store)
    }

    /// Calls the plugin's `get_version()` export.
    #[allow(dead_code)] // Used by plugin discovery UI in later phases
    pub fn call_get_version(&mut self) -> wasmtime::Result<String> {
        self.bindings.call_get_version(&mut self.store)
    }

    /// Calls the plugin's `get_author()` export.
    #[allow(dead_code)] // Used by plugin discovery UI in later phases
    pub fn call_get_author(&mut self) -> wasmtime::Result<String> {
        self.bindings.call_get_author(&mut self.store)
    }

    /// Drains the accumulated render commands from the current tick,
    /// returning them as a `Vec<RenderCommand>` and clearing the buffer.
    pub fn take_frame(&mut self) -> Vec<RenderCommand> {
        self.store.data_mut().frame_buffer.drain(..).collect()
    }

    /// Takes the game-over score if one was signalled this frame.
    /// Returns `Some(score)` once and resets the flag.
    pub fn take_game_over_score(&mut self) -> Option<u32> {
        self.store.data_mut().game_over_score.take()
    }
}
