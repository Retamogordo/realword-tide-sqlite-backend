use sqlx::Connection;
use sqlx::{SqliteConnection, Pool};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, Sqlite};
use std::str::FromStr;
use sqlx::Row;

#[derive(sqlx::FromRow)]
pub(crate) struct UserDB {
    email: String,    
    username: String,    
    password: String,
}

pub async fn connect(database_url_prefix: &str, database_url_path: &str, 
    database_file: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
//    let conn = SqliteConnection::connect("sqlite::memory:").await?;

//    let database_url = "sqlite:///home/yury/sqlite/my_test.db";
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
    
 //   sqlx::migrate!("../../../sqlite").run(&sqlite_pool).await?;

    sqlx::query("
        DROP TABLE IF EXISTS users;
        CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL
        );
    ")
    .execute(&sqlite_pool)    
    .await?;
    
    sqlx::query("
        DROP TABLE IF EXISTS profiles;
        CREATE TABLE IF NOT EXISTS profiles (
            username TEXT NOT NULL,
            bio TEXT,
            image TEXT,
            following BIT,
            FOREIGN KEY (username)
               REFERENCES users (username) 
               ON DELETE CASCADE
               ON UPDATE CASCADE
        );
    ")
    .execute(&sqlite_pool)    
    .await?;

    Ok(sqlite_pool)
}

pub(crate) async fn register_user(conn: &Pool<Sqlite>,
    user: &crate::UserReg,
) -> Result<(), crate::errors::RegistrationError>  {
    sqlx::query(
            "INSERT INTO users (username, email, password)
            VALUES( ?,	?, ?);\n
            INSERT INTO profiles (username)
            VALUES( ?);
            ")
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.username)
        .execute(conn)    
        .await?;
    Ok(())
}

pub(crate) async fn get_user(conn: &Pool<Sqlite>,
    email: &str,
) -> Result<crate::User, crate::errors::RegistrationError>  {

    let user: crate::User = sqlx::query_as::<_, crate::User>(
            "SELECT *, NULL as `token` FROM users LEFT JOIN profiles ON users.username = profiles.username WHERE email = ?;")
        .bind(email)
        .fetch_one(conn)   
        .await?;
    Ok(user)
}

pub(crate) async fn get_profile(conn: &Pool<Sqlite>,
    username: &str,
) -> Result<crate::Profile, crate::errors::RegistrationError>  {

    let profile: crate::Profile = sqlx::query_as::<_, crate::Profile>(
            "SELECT * FROM profiles INNER JOIN users ON profiles.username = users.username WHERE profiles.username = ?;")
        .bind(username)
        .fetch_one(conn)   
        .await?;
    Ok(profile)
}
