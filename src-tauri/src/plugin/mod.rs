pub mod discovery;

use std::path::Path;

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
/// Provides WASI context, resource table, and a per-frame draw command buffer.
struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
    /// Accumulated render commands from the current tick's host import calls.
    frame_buffer: Vec<RenderCommand>,
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
        self.frame_buffer.push(RenderCommand::DrawRect { x, y, w, h, color });
    }

    fn draw_text(&mut self, text: String, x: f32, y: f32, size: u8) {
        self.frame_buffer.push(RenderCommand::DrawText { text, x, y, size });
    }

    fn play_sound(&mut self, id: u32) {
        log::info!("[stub] play_sound(id={id})");
    }

    fn get_storage(&mut self, key: String) -> Option<String> {
        log::info!("[stub] get_storage(key={key:?})");
        None
    }

    fn set_storage(&mut self, key: String, value: String) {
        log::info!("[stub] set_storage(key={key:?}, value={value:?})");
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
    /// # Errors
    /// Returns a wasmtime error if the file cannot be read, the component
    /// is invalid, or imports cannot be satisfied.
    pub fn load(wasm_path: &Path) -> wasmtime::Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;

        let component = Component::from_file(&engine, wasm_path)?;

        let mut linker: Linker<HostState> = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync::<HostState>(&mut linker)?;
        IgnisGame::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

        let wasi = WasiCtxBuilder::new().build();
        let mut store = Store::new(
            &engine,
            HostState {
                wasi,
                table: ResourceTable::new(),
                frame_buffer: Vec::new(),
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
    pub fn call_get_name(&mut self) -> wasmtime::Result<String> {
        self.bindings.call_get_name(&mut self.store)
    }

    /// Calls the plugin's `get_version()` export.
    pub fn call_get_version(&mut self) -> wasmtime::Result<String> {
        self.bindings.call_get_version(&mut self.store)
    }

    /// Calls the plugin's `get_author()` export.
    pub fn call_get_author(&mut self) -> wasmtime::Result<String> {
        self.bindings.call_get_author(&mut self.store)
    }

    /// Drains the accumulated render commands from the current tick,
    /// returning them as a `Vec<RenderCommand>` and clearing the buffer.
    pub fn take_frame(&mut self) -> Vec<RenderCommand> {
        self.store.data_mut().frame_buffer.drain(..).collect()
    }
}
