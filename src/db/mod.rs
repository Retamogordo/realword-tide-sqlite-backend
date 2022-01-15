//use sqlx::Connection;
use sqlx::{Pool};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, Sqlite};
use std::str::FromStr;
use crate::app;

pub(crate) mod article;
pub(crate) mod user;
pub(crate) mod schema;

pub(crate) async fn connect(config: &app::Config) -> Result<Pool<Sqlite>, sqlx::Error> {
    let sqlite_pool = schema::Schema::with_config(&config)?.create().await?;

    Ok(sqlite_pool)
}


