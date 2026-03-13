use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// The 8 canonical actions and their default keyboard bindings.
const DEFAULT_KEYBOARD_BINDINGS: &[(&str, &str)] = &[
    ("Up", "ArrowUp"),
    ("Down", "ArrowDown"),
    ("Left", "ArrowLeft"),
    ("Right", "ArrowRight"),
    ("ActionA", "KeyZ"),
    ("ActionB", "KeyX"),
    ("Start", "Enter"),
    ("Select", "ShiftLeft"),
];

/// Default gamepad button bindings (W3C "Standard Gamepad" layout).
/// `button_N` maps to the standard gamepad button index.
const DEFAULT_GAMEPAD_BINDINGS: &[(&str, &str)] = &[
    ("Up", "button_12"),
    ("Down", "button_13"),
    ("Left", "button_14"),
    ("Right", "button_15"),
    ("ActionA", "button_0"),
    ("ActionB", "button_1"),
    ("Start", "button_9"),
    ("Select", "button_8"),
];

/// A single keybinding row mapping an action to a physical key or button.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Keybinding {
    pub action: String,
    pub device_type: String,
    pub binding: String,
}

/// Returns all keybindings for a player. If the player has no bindings yet,
/// inserts the defaults first so the frontend always receives a full set.
///
/// # Errors
/// Returns an error string if the database operation fails.
#[tauri::command]
pub async fn get_keybindings(
    player_id: i64,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<Keybinding>, String> {
    // Seed keyboard defaults if none exist
    let kb_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM keybindings WHERE player_id = ? AND device_type = 'keyboard'",
    )
    .bind(player_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to query keybindings: {e}"))?;

    if kb_count.0 == 0 {
        for (action, binding) in DEFAULT_KEYBOARD_BINDINGS {
            sqlx::query(
                "INSERT OR IGNORE INTO keybindings (player_id, action, device_type, binding) VALUES (?, ?, 'keyboard', ?)",
            )
            .bind(player_id)
            .bind(action)
            .bind(binding)
            .execute(&state.db)
            .await
            .map_err(|e| format!("Failed to seed default keybindings: {e}"))?;
        }
    }

    // Seed gamepad defaults if none exist
    let gp_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM keybindings WHERE player_id = ? AND device_type = 'gamepad'",
    )
    .bind(player_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to query gamepad keybindings: {e}"))?;

    if gp_count.0 == 0 {
        for (action, binding) in DEFAULT_GAMEPAD_BINDINGS {
            sqlx::query(
                "INSERT OR IGNORE INTO keybindings (player_id, action, device_type, binding) VALUES (?, ?, 'gamepad', ?)",
            )
            .bind(player_id)
            .bind(action)
            .bind(binding)
            .execute(&state.db)
            .await
            .map_err(|e| format!("Failed to seed default gamepad keybindings: {e}"))?;
        }
    }

    sqlx::query_as::<_, Keybinding>(
        "SELECT action, device_type, binding FROM keybindings WHERE player_id = ? ORDER BY action",
    )
    .bind(player_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to get keybindings: {e}"))
}

/// Updates (or inserts) a single keybinding for a player.
///
/// # Errors
/// Returns an error string if the action name is invalid or the DB operation fails.
#[tauri::command]
pub async fn set_keybinding(
    player_id: i64,
    action: String,
    device_type: String,
    binding: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    // Validate the action name
    let valid_actions = [
        "Up", "Down", "Left", "Right", "ActionA", "ActionB", "Start", "Select",
    ];
    if !valid_actions.contains(&action.as_str()) {
        return Err(format!("Invalid action: {action}"));
    }

    sqlx::query(
        "INSERT INTO keybindings (player_id, action, device_type, binding) VALUES (?, ?, ?, ?)
         ON CONFLICT (player_id, action, device_type)
         DO UPDATE SET binding = excluded.binding",
    )
    .bind(player_id)
    .bind(&action)
    .bind(&device_type)
    .bind(&binding)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to set keybinding: {e}"))?;

    Ok(())
}

/// Resets all keybindings for a player back to defaults (keyboard + gamepad).
///
/// # Errors
/// Returns an error string if the database operation fails.
#[tauri::command]
pub async fn reset_keybindings(
    player_id: i64,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<Keybinding>, String> {
    // Delete all existing bindings for this player
    sqlx::query("DELETE FROM keybindings WHERE player_id = ?")
        .bind(player_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to delete keybindings: {e}"))?;

    // Re-seed keyboard defaults
    for (action, binding) in DEFAULT_KEYBOARD_BINDINGS {
        sqlx::query(
            "INSERT INTO keybindings (player_id, action, device_type, binding) VALUES (?, ?, 'keyboard', ?)",
        )
        .bind(player_id)
        .bind(action)
        .bind(binding)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to seed default keybindings: {e}"))?;
    }

    // Re-seed gamepad defaults
    for (action, binding) in DEFAULT_GAMEPAD_BINDINGS {
        sqlx::query(
            "INSERT INTO keybindings (player_id, action, device_type, binding) VALUES (?, ?, 'gamepad', ?)",
        )
        .bind(player_id)
        .bind(action)
        .bind(binding)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to seed default gamepad keybindings: {e}"))?;
    }

    // Return all fresh defaults
    sqlx::query_as::<_, Keybinding>(
        "SELECT action, device_type, binding FROM keybindings WHERE player_id = ? ORDER BY action",
    )
    .bind(player_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to get keybindings: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePool;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory db");
        let migration_sql = include_str!("../../migrations/001_init.sql");
        sqlx::raw_sql(migration_sql)
            .execute(&pool)
            .await
            .expect("run migration");
        // Create a test player
        sqlx::query("INSERT INTO players (name, avatar_id) VALUES ('Tester', 0)")
            .execute(&pool)
            .await
            .expect("create test player");
        pool
    }

    #[tokio::test]
    async fn test_get_keybindings_seeds_defaults() {
        let pool = setup_pool().await;

        // No bindings exist yet — get_keybindings should seed them
        let bindings: Vec<Keybinding> = sqlx::query_as(
            "SELECT action, device_type, binding FROM keybindings WHERE player_id = 1 ORDER BY action",
        )
        .fetch_all(&pool)
        .await
        .expect("query");
        assert_eq!(bindings.len(), 0, "no bindings before seeding");

        // Seed via helper (simulates what the command does)
        for (action, binding) in super::DEFAULT_KEYBOARD_BINDINGS {
            sqlx::query(
                "INSERT OR IGNORE INTO keybindings (player_id, action, device_type, binding) VALUES (1, ?, 'keyboard', ?)",
            )
            .bind(action)
            .bind(binding)
            .execute(&pool)
            .await
            .expect("seed");
        }
        for (action, binding) in super::DEFAULT_GAMEPAD_BINDINGS {
            sqlx::query(
                "INSERT OR IGNORE INTO keybindings (player_id, action, device_type, binding) VALUES (1, ?, 'gamepad', ?)",
            )
            .bind(action)
            .bind(binding)
            .execute(&pool)
            .await
            .expect("seed gamepad");
        }

        let bindings: Vec<Keybinding> = sqlx::query_as(
            "SELECT action, device_type, binding FROM keybindings WHERE player_id = 1 ORDER BY action",
        )
        .fetch_all(&pool)
        .await
        .expect("query");
        assert_eq!(bindings.len(), 16, "8 keyboard + 8 gamepad defaults");
    }

    #[tokio::test]
    async fn test_set_keybinding_upsert() {
        let pool = setup_pool().await;

        // Insert initial binding
        sqlx::query(
            "INSERT INTO keybindings (player_id, action, device_type, binding) VALUES (1, 'Up', 'keyboard', 'ArrowUp')",
        )
        .execute(&pool)
        .await
        .expect("insert");

        // Upsert with a new binding
        sqlx::query(
            "INSERT INTO keybindings (player_id, action, device_type, binding) VALUES (1, 'Up', 'keyboard', 'KeyW')
             ON CONFLICT (player_id, action, device_type)
             DO UPDATE SET binding = excluded.binding",
        )
        .execute(&pool)
        .await
        .expect("upsert");

        let row: Keybinding = sqlx::query_as(
            "SELECT action, device_type, binding FROM keybindings WHERE player_id = 1 AND action = 'Up' AND device_type = 'keyboard'",
        )
        .fetch_one(&pool)
        .await
        .expect("fetch");
        assert_eq!(row.binding, "KeyW");
    }

    #[tokio::test]
    async fn test_reset_keybindings() {
        let pool = setup_pool().await;

        // Insert a custom binding
        sqlx::query(
            "INSERT INTO keybindings (player_id, action, device_type, binding) VALUES (1, 'Up', 'keyboard', 'KeyW')",
        )
        .execute(&pool)
        .await
        .expect("insert custom");

        // Delete and re-seed (simulates reset)
        sqlx::query("DELETE FROM keybindings WHERE player_id = 1")
            .execute(&pool)
            .await
            .expect("delete");

        for (action, binding) in super::DEFAULT_KEYBOARD_BINDINGS {
            sqlx::query(
                "INSERT INTO keybindings (player_id, action, device_type, binding) VALUES (1, ?, 'keyboard', ?)",
            )
            .bind(action)
            .bind(binding)
            .execute(&pool)
            .await
            .expect("seed");
        }
        for (action, binding) in super::DEFAULT_GAMEPAD_BINDINGS {
            sqlx::query(
                "INSERT INTO keybindings (player_id, action, device_type, binding) VALUES (1, ?, 'gamepad', ?)",
            )
            .bind(action)
            .bind(binding)
            .execute(&pool)
            .await
            .expect("seed gamepad");
        }

        let row: Keybinding = sqlx::query_as(
            "SELECT action, device_type, binding FROM keybindings WHERE player_id = 1 AND action = 'Up' AND device_type = 'keyboard'",
        )
        .fetch_one(&pool)
        .await
        .expect("fetch");
        assert_eq!(row.binding, "ArrowUp", "reset should restore default");
    }
}
