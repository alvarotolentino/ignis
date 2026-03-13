mod db;
mod engine;
mod input;
mod plugin;

use std::sync::Arc;

use sqlx::sqlite::SqlitePool;
use tauri::Manager;

use crate::engine::GameEngine;

/// Application state holding the database pool and game engine.
pub struct AppState {
    pub db: SqlitePool,
    pub engine: Arc<GameEngine>,
}

/// Resolves the plugins directory. Tries resource_dir/plugins first (release),
/// then falls back to `../plugins` relative to the executable (dev).
fn resolve_plugins_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    // In release builds, resources are bundled alongside the executable
    if let Ok(res) = app.path().resource_dir() {
        let candidate = res.join("plugins");
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    // Dev fallback: plugins/ at the workspace root (parent of src-tauri/)
    let exe = std::env::current_exe().map_err(|e| format!("Cannot find exe: {e}"))?;
    if let Some(dir) = exe.parent() {
        // In dev, exe is in src-tauri/target/debug/
        for ancestor in [dir, dir.parent().unwrap_or(dir)] {
            let candidate = ancestor.join("plugins");
            if candidate.is_dir() {
                return Ok(candidate);
            }
        }
    }

    // Last resort: check relative to CWD (cargo tauri dev runs from src-tauri/)
    let cwd_candidate = std::path::PathBuf::from("../plugins");
    if cwd_candidate.is_dir() {
        return cwd_candidate.canonicalize().map_err(|e| e.to_string());
    }

    Err("Cannot find plugins directory".into())
}

/// Starts a game by loading the plugin matching `plugin_id` and launching the 60 Hz loop.
#[tauri::command]
fn start_game(
    plugin_id: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let plugins_dir = resolve_plugins_dir(&app)?;

    let plugins = plugin::discovery::discover_plugins(&plugins_dir);
    let found = plugins
        .into_iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| format!("Plugin '{plugin_id}' not found"))?;

    state.engine.start_game(found.wasm_path, &plugin_id, app)
}

/// Stops the currently running game loop.
#[tauri::command]
fn stop_game(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.engine.stop_game();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize SQLite database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");

            let pool = tauri::async_runtime::block_on(db::init_db(&app_data_dir))
                .expect("failed to initialize database");

            let engine = Arc::new(GameEngine::new());

            // Set the DB path so plugins can use get_storage / set_storage
            let db_path = app_data_dir.join("ignis.db");
            *engine.db_path.lock().unwrap() = Some(db_path);

            app.manage(AppState { db: pool, engine });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            db::players::create_player,
            db::players::list_players,
            db::players::delete_player,
            db::scores::submit_score,
            db::scores::get_high_scores,
            db::keybindings::get_keybindings,
            db::keybindings::set_keybinding,
            db::keybindings::reset_keybindings,
            plugin::discovery::list_discovered_plugins,
            start_game,
            stop_game,
            input::send_input,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
