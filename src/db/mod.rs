use sqlx::Connection;
use sqlx::{Pool};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, Sqlite};
use std::str::FromStr;
use sqlx::Row;

pub(crate) mod article;
pub(crate) mod user;
pub(crate) mod schema;

pub async fn connect(database_url_prefix: &str, database_url_path: &str, 
    database_file: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
//    let conn = SqliteConnection::connect("sqlite::memory:").await?;

    match async_std::fs::create_dir(database_url_path).await {
        Ok(()) => Ok(()),
        Err(err) if async_std::io::ErrorKind::AlreadyExists == err.kind() => Ok(()),
        err @ _ => err, 
    }?;

    let database_url = format!("{}{}", database_url_prefix, database_url_path);

    let connection_options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .filename(database_file);
//        .journal_mode(SqliteJournalMode::Wal)
//        .synchronous(SqliteSynchronous::Normal)
//        .busy_timeout(pool_timeout);

    let sqlite_pool = SqlitePoolOptions::new()
//        .max_connections(pool_max_connections)
//        .connect_timeout(pool_timeout)
        .connect_with(connection_options)
        .await?;

    schema::create(&sqlite_pool).await?;

    Ok(sqlite_pool)
    
 //   sqlx::migrate!("../../../sqlite").run(&sqlite_pool).await?;

}


