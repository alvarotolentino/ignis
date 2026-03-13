pub mod keybindings;
pub mod players;
pub mod scores;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

/// Initialize the SQLite database, run migrations, and return the connection pool.
pub async fn init_db(app_data_dir: &Path) -> Result<SqlitePool, sqlx::Error> {
    std::fs::create_dir_all(app_data_dir).ok();
    let db_path = app_data_dir.join("ignis.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run the init migration
    let migration_sql = include_str!("../../migrations/001_init.sql");
    sqlx::raw_sql(migration_sql).execute(&pool).await?;

    log::info!("Database initialized at {}", db_path.display());
    Ok(pool)
}
