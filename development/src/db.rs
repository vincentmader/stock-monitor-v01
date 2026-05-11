use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};

pub type Db = PgPool;

pub async fn connect(database_url: &str) -> Result<Db> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("failed to connect to PostgreSQL")?;

    tracing::info!("database connection pool established");
    Ok(pool)
}
