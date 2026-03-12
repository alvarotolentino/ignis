use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::FromRow;

/// A player profile stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Player {
    pub id: i64,
    pub name: String,
    pub avatar_id: i32,
    pub created_at: String,
}

/// Creates a new player profile.
///
/// # Errors
/// Returns an error string if the name is empty or already exists.
#[tauri::command]
pub async fn create_player(
    name: String,
    avatar_id: i32,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Player, String> {
    if name.trim().is_empty() {
        return Err("Player name cannot be empty".into());
    }

    let result = sqlx::query_as::<_, Player>(
        "INSERT INTO players (name, avatar_id) VALUES (?, ?) RETURNING id, name, avatar_id, created_at",
    )
    .bind(name.trim())
    .bind(avatar_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Failed to create player: {e}"))?;

    Ok(result)
}

/// Lists all player profiles ordered by creation date.
#[tauri::command]
pub async fn list_players(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<Player>, String> {
    sqlx::query_as::<_, Player>("SELECT id, name, avatar_id, created_at FROM players ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| format!("Failed to list players: {e}"))
}

/// Deletes a player profile by ID.
#[tauri::command]
pub async fn delete_player(
    id: i64,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM players WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to delete player: {e}"))?;

    Ok(())
}

pub fn commands() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![create_player, list_players, delete_player]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_player_crud() {
        // Create an in-memory SQLite database for testing
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory db");

        let migration_sql = include_str!("../../migrations/001_init.sql");
        sqlx::raw_sql(migration_sql)
            .execute(&pool)
            .await
            .expect("run migration");

        // Create a player
        let player = sqlx::query_as::<_, Player>(
            "INSERT INTO players (name, avatar_id) VALUES (?, ?) RETURNING id, name, avatar_id, created_at",
        )
        .bind("TestPlayer")
        .bind(2)
        .fetch_one(&pool)
        .await
        .expect("create player");

        assert_eq!(player.name, "TestPlayer");
        assert_eq!(player.avatar_id, 2);
        assert!(player.id > 0);

        // List players
        let players = sqlx::query_as::<_, Player>(
            "SELECT id, name, avatar_id, created_at FROM players ORDER BY created_at DESC",
        )
        .fetch_all(&pool)
        .await
        .expect("list players");

        assert_eq!(players.len(), 1);
        assert_eq!(players[0].name, "TestPlayer");

        // Delete player
        sqlx::query("DELETE FROM players WHERE id = ?")
            .bind(player.id)
            .execute(&pool)
            .await
            .expect("delete player");

        let players = sqlx::query_as::<_, Player>(
            "SELECT id, name, avatar_id, created_at FROM players ORDER BY created_at DESC",
        )
        .fetch_all(&pool)
        .await
        .expect("list players after delete");

        assert_eq!(players.len(), 0);
    }
}
