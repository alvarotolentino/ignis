pub mod types;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::Emitter;

use crate::engine::types::{InputEvent, RenderFrame};
use crate::plugin::PluginRuntime;

/// Sprite sheet data sent to the frontend for loading.
#[derive(Clone, Debug, Serialize)]
pub struct SpriteSheetPayload {
    /// Base64-encoded PNG image data.
    pub image_base64: String,
    /// JSON string describing frame coordinates.
    pub meta_json: String,
}

/// A single sound asset sent to the frontend for loading.
#[derive(Clone, Debug, Serialize)]
pub struct SoundAsset {
    pub id: u32,
    pub data_base64: String,
    pub mime: String,
}

/// Collection of sound assets for a plugin.
#[derive(Clone, Debug, Serialize)]
pub struct SoundsPayload {
    pub sounds: Vec<SoundAsset>,
}

/// Payload emitted when a plugin signals game over via `set_storage("last_score", ...)`.
#[derive(Clone, Debug, Serialize)]
pub struct GameOverPayload {
    pub game_id: String,
    pub score: u32,
}

/// Thread-safe game engine managing the active plugin and game loop.
pub struct GameEngine {
    /// Whether the game loop is currently running.
    running: Arc<AtomicBool>,
    /// The active WASM plugin instance (behind Mutex for Send).
    active_plugin: Arc<Mutex<Option<PluginRuntime>>>,
    /// Queue of input events from the frontend, drained each tick.
    pub input_queue: Arc<Mutex<VecDeque<InputEvent>>>,
    /// Path to the SQLite database file used for plugin storage.
    pub db_path: Arc<Mutex<Option<std::path::PathBuf>>>,
}

impl GameEngine {
    /// Creates a new idle `GameEngine` with no active plugin.
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            active_plugin: Arc::new(Mutex::new(None)),
            input_queue: Arc::new(Mutex::new(VecDeque::new())),
            db_path: Arc::new(Mutex::new(None)),
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
        plugin_id: &str,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        // Stop any existing game loop first
        self.stop_game();

        let db_path = self
            .db_path
            .lock()
            .unwrap()
            .clone()
            .ok_or("Database path not set")?;

        let mut runtime = PluginRuntime::load(&wasm_path, plugin_id, &db_path)
            .map_err(|e| format!("Failed to load plugin: {e}"))?;

        runtime
            .call_init()
            .map_err(|e| format!("Plugin init() failed: {e}"))?;

        log::info!("Plugin loaded and initialized from {}", wasm_path.display());

        // Load and emit sprite sheet if the plugin has one
        if let Some(plugin_dir) = wasm_path.parent() {
            self.load_sprite_sheet(plugin_dir, &app_handle);
            self.load_sounds(plugin_dir, &app_handle);
        }

        *self.active_plugin.lock().unwrap() = Some(runtime);
        self.running.store(true, Ordering::SeqCst);

        // Clone Arcs for the spawned task
        let running = Arc::clone(&self.running);
        let plugin = Arc::clone(&self.active_plugin);
        let input_q = Arc::clone(&self.input_queue);
        let game_id = plugin_id.to_string();

        tauri::async_runtime::spawn(async move {
            use tokio::time::{interval, Duration};

            let mut ticker = interval(Duration::from_micros(16_667)); // ~60 Hz

            while running.load(Ordering::SeqCst) {
                ticker.tick().await;

                let (frame, game_over_score) = {
                    let mut guard = plugin.lock().unwrap();
                    if let Some(rt) = guard.as_mut() {
                        // Drain input queue and dispatch to plugin.
                        // Encode press/release into the action u32:
                        //   0–7  = press events
                        //   8–15 = release events (action + 8)
                        // Backward-compatible: plugins that only match 0–7
                        // silently ignore releases via their `_ => {}` arm.
                        {
                            let mut q = input_q.lock().unwrap();
                            while let Some(ev) = q.pop_front() {
                                let action_id = if ev.pressed {
                                    ev.action.to_u32()
                                } else {
                                    ev.action.to_u32() + 8
                                };
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
                        let frame = RenderFrame {
                            commands: rt.take_frame(),
                        };

                        // Check if plugin signalled game over
                        let go_score = rt.take_game_over_score();
                        (frame, go_score)
                    } else {
                        (RenderFrame::default(), None)
                    }
                };

                // Emit render frame
                let _ = app_handle.emit("render_frame", &frame);

                // Emit game_over if the plugin signalled it
                if let Some(score) = game_over_score {
                    let payload = GameOverPayload {
                        game_id: game_id.clone(),
                        score,
                    };
                    let _ = app_handle.emit("game_over", &payload);
                }
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

    /// Reads the sprite sheet from a plugin's `sprites/` directory and emits it to the frontend.
    fn load_sprite_sheet(&self, plugin_dir: &std::path::Path, app_handle: &tauri::AppHandle) {
        let sprites_dir = plugin_dir.join("sprites");
        let json_path = sprites_dir.join("spritesheet.json");
        let png_path = sprites_dir.join("spritesheet.png");

        if !json_path.exists() || !png_path.exists() {
            log::info!("No sprite sheet found in {}", sprites_dir.display());
            return;
        }

        let meta_json = match std::fs::read_to_string(&json_path) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to read spritesheet.json: {e}");
                return;
            }
        };

        let png_bytes = match std::fs::read(&png_path) {
            Ok(b) => b,
            Err(e) => {
                log::warn!("Failed to read spritesheet.png: {e}");
                return;
            }
        };

        use base64::Engine as _;
        let image_base64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

        let payload = SpriteSheetPayload {
            image_base64,
            meta_json,
        };

        if let Err(e) = app_handle.emit("load_sprite_sheet", &payload) {
            log::warn!("Failed to emit sprite sheet: {e}");
        } else {
            log::info!("Sprite sheet emitted from {}", sprites_dir.display());
        }
    }

    /// Reads sound files from a plugin's `sounds/` directory and emits them to the frontend.
    fn load_sounds(&self, plugin_dir: &std::path::Path, app_handle: &tauri::AppHandle) {
        let sounds_dir = plugin_dir.join("sounds");
        if !sounds_dir.is_dir() {
            log::info!("No sounds directory in {}", plugin_dir.display());
            return;
        }

        let mut sounds = Vec::new();

        let entries = match std::fs::read_dir(&sounds_dir) {
            Ok(e) => e,
            Err(e) => {
                log::warn!("Failed to read sounds directory: {e}");
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Ok(id) = stem.parse::<u32>() else {
                continue;
            };

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let mime = match ext {
                "wav" => "audio/wav",
                "mp3" => "audio/mpeg",
                "ogg" => "audio/ogg",
                _ => continue,
            };

            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(e) => {
                    log::warn!("Failed to read sound {}: {e}", path.display());
                    continue;
                }
            };

            use base64::Engine as _;
            let data_base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

            sounds.push(SoundAsset {
                id,
                data_base64,
                mime: mime.to_string(),
            });
        }

        if sounds.is_empty() {
            return;
        }

        let payload = SoundsPayload { sounds };
        if let Err(e) = app_handle.emit("load_sounds", &payload) {
            log::warn!("Failed to emit sounds: {e}");
        } else {
            log::info!("Sounds emitted from {}", sounds_dir.display());
        }
    }
}