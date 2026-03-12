pub mod types;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::Emitter;

use crate::engine::types::{InputEvent, RenderFrame};
use crate::plugin::PluginRuntime;

/// Thread-safe game engine managing the active plugin and game loop.
pub struct GameEngine {
    /// Whether the game loop is currently running.
    running: Arc<AtomicBool>,
    /// The active WASM plugin instance (behind Mutex for Send).
    active_plugin: Arc<Mutex<Option<PluginRuntime>>>,
    /// Queue of input events from the frontend, drained each tick.
    pub input_queue: Arc<Mutex<VecDeque<InputEvent>>>,
}

impl GameEngine {
    /// Creates a new idle `GameEngine` with no active plugin.
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            active_plugin: Arc::new(Mutex::new(None)),
            input_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Loads a WASM plugin from disk, calls its `init()` export,
    /// and starts the 60 Hz game loop that emits `render_frame` events.
    ///
    /// # Errors
    /// Returns an error string if the plugin fails to load or init.
    pub fn start_game(
        &self,
        wasm_path: PathBuf,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        // Stop any existing game loop first
        self.stop_game();

        let mut runtime =
            PluginRuntime::load(&wasm_path).map_err(|e| format!("Failed to load plugin: {e}"))?;

        runtime
            .call_init()
            .map_err(|e| format!("Plugin init() failed: {e}"))?;

        log::info!("Plugin loaded and initialized from {}", wasm_path.display());

        *self.active_plugin.lock().unwrap() = Some(runtime);
        self.running.store(true, Ordering::SeqCst);

        // Clone Arcs for the spawned task
        let running = Arc::clone(&self.running);
        let plugin = Arc::clone(&self.active_plugin);
        let input_q = Arc::clone(&self.input_queue);

        tauri::async_runtime::spawn(async move {
            use tokio::time::{interval, Duration};

            let mut ticker = interval(Duration::from_micros(16_667)); // ~60 Hz

            while running.load(Ordering::SeqCst) {
                ticker.tick().await;

                let frame = {
                    let mut guard = plugin.lock().unwrap();
                    if let Some(rt) = guard.as_mut() {
                        // Drain input queue and dispatch to plugin
                        {
                            let mut q = input_q.lock().unwrap();
                            while let Some(ev) = q.pop_front() {
                                let action_id = ev.action.to_u32();
                                if let Err(e) = rt.call_handle_input(action_id) {
                                    log::warn!("handle_input failed: {e}");
                                }
                            }
                        }

                        // Tick the plugin
                        if let Err(e) = rt.call_update(16) {
                            log::warn!("update() failed: {e}");
                        }

                        // Collect the frame
                        RenderFrame {
                            commands: rt.take_frame(),
                        }
                    } else {
                        RenderFrame::default()
                    }
                };

                // Emit to frontend — ignore errors (e.g., no listeners)
                let _ = app_handle.emit("render_frame", &frame);
            }

            log::info!("Game loop stopped");
        });

        Ok(())
    }

    /// Stops the game loop and drops the active plugin.
    pub fn stop_game(&self) {
        self.running.store(false, Ordering::SeqCst);
        *self.active_plugin.lock().unwrap() = None;
    }
}