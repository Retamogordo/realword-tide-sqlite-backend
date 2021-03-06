use sqlx::{Pool};
use sqlx::sqlite::{Sqlite};
use crate::config;

pub(crate) mod article;
pub(crate) mod user;
pub(crate) mod schema;

pub(crate) async fn connect(config: &config::Config) -> Result<Pool<Sqlite>, sqlx::Error> {
    let sqlite_pool = schema::Schema::with_config(&config)?.create().await?;

    Ok(sqlite_pool)
}


