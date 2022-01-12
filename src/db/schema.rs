use sqlx::{Pool};
use sqlx::sqlite::{Sqlite, SqlitePool};

pub(crate) async fn create (sqlite_pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("
        DROP TABLE IF EXISTS users;
        CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL
        );
    ")
    .execute(sqlite_pool)    
    .await?;
    
    sqlx::query("
        DROP TABLE IF EXISTS profiles;
        CREATE TABLE IF NOT EXISTS profiles (
            username TEXT NOT NULL,
            bio TEXT,
            image TEXT,
            FOREIGN KEY (username)
               REFERENCES users (username) 
               ON DELETE CASCADE
               ON UPDATE CASCADE
        );
    ")
    .execute(sqlite_pool)    
    .await?;

    sqlx::query("
        DROP TABLE IF EXISTS followers;
        CREATE TABLE IF NOT EXISTS followers (
            follower_name TEXT NOT NULL,
            celeb_name TEXT NOT NULL,
            FOREIGN KEY (celeb_name)
                REFERENCES users (username) 
                ON DELETE CASCADE
                ON UPDATE CASCADE,
            FOREIGN KEY (follower_name)
                REFERENCES users (username) 
                ON DELETE CASCADE
                ON UPDATE CASCADE
            CONSTRAINT Pair UNIQUE (follower_name,celeb_name)
        );
    ")
    .execute(sqlite_pool)    
    .await?;

    sqlx::query("
        DROP TABLE IF EXISTS articles;
        CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            slug TEXT,
            title TEXT NOT NULL,
            description TEXT,
            body TEXT NOT NULL,
            tagList TEXT,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL,
            author TEXT NOT NULL   
        );
    ")
    .execute(sqlite_pool)    
    .await?;
    
    sqlx::query("
        DROP TABLE IF EXISTS favorite_articles;
        CREATE TABLE IF NOT EXISTS favorite_articles (
            id INTEGER NOT NULL,
            username TEXT NOT NULL,
            FOREIGN KEY (id)
                REFERENCES articles (id) 
                ON DELETE CASCADE
            FOREIGN KEY (username)
                REFERENCES users (username) 
                ON DELETE CASCADE
                ON UPDATE CASCADE
            CONSTRAINT Pair UNIQUE (id, username)
        );
    ")
    .execute(sqlite_pool)    
    .await?;

    sqlx::query("
        DROP TABLE IF EXISTS comments;
        CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            article_id INTEGER NOT NULL,
            body TEXT NOT NULL,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL,
            author TEXT NOT NULL,   
        FOREIGN KEY (article_id)
            REFERENCES articles (id) 
            ON DELETE CASCADE
        FOREIGN KEY (author)
            REFERENCES users (username) 
            ON DELETE CASCADE
            ON UPDATE CASCADE
        );
    ")
    .execute(sqlite_pool)    
    .await?;

    Ok(())
}