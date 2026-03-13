use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A score entry with the player name, for display in high score tables.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScoreEntry {
    pub player_name: String,
    pub score: i64,
    pub achieved_at: String,
}

/// Submits a score for a player in a specific game.
///
/// # Errors
/// Returns an error string if the database operation fails.
#[tauri::command]
pub async fn submit_score(
    player_id: i64,
    game_id: String,
    score: i64,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    sqlx::query("INSERT INTO scores (player_id, game_id, score) VALUES (?, ?, ?)")
        .bind(player_id)
        .bind(&game_id)
        .bind(score)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to submit score: {e}"))?;

    Ok(())
}

/// Returns the top N high scores for a given game, joined with player names.
///
/// # Errors
/// Returns an error string if the database operation fails.
#[tauri::command]
pub async fn get_high_scores(
    game_id: String,
    limit: i64,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<ScoreEntry>, String> {
    let limit = limit.clamp(1, 100);
    sqlx::query_as::<_, ScoreEntry>(
        "SELECT p.name AS player_name, s.score, s.achieved_at
         FROM scores s
         JOIN players p ON p.id = s.player_id
         WHERE s.game_id = ?
         ORDER BY s.score DESC
         LIMIT ?",
    )
    .bind(&game_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to get high scores: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePool;

    #[tokio::test]
    async fn test_submit_and_get_high_scores() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory db");

        let migration_sql = include_str!("../../migrations/001_init.sql");
        sqlx::raw_sql(migration_sql)
            .execute(&pool)
            .await
            .expect("run migration");

        // Create a player
        let player: (i64,) =
            sqlx::query_as("INSERT INTO players (name, avatar_id) VALUES (?, ?) RETURNING id")
                .bind("Alice")
                .bind(0)
                .fetch_one(&pool)
                .await
                .expect("create player");

        let player_id = player.0;

        // Submit scores
        for score in [500, 1200, 300, 800] {
            sqlx::query("INSERT INTO scores (player_id, game_id, score) VALUES (?, ?, ?)")
                .bind(player_id)
                .bind("space-invaders")
                .bind(score)
                .execute(&pool)
                .await
                .expect("submit score");
        }

        // Get top 3
        let top = sqlx::query_as::<_, ScoreEntry>(
            "SELECT p.name AS player_name, s.score, s.achieved_at
             FROM scores s
             JOIN players p ON p.id = s.player_id
             WHERE s.game_id = ?
             ORDER BY s.score DESC
             LIMIT ?",
        )
        .bind("space-invaders")
        .bind(3i64)
        .fetch_all(&pool)
        .await
        .expect("get high scores");

        assert_eq!(top.len(), 3);
        assert_eq!(top[0].score, 1200);
        assert_eq!(top[1].score, 800);
        assert_eq!(top[2].score, 500);
        assert_eq!(top[0].player_name, "Alice");
    }
}
