use sqlx::sqlite::{SqlitePool};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use crate::app;

pub(crate) struct Schema {
//    sqlite_pool: &'a SqlitePool,
    connection_options: SqliteConnectOptions,
}

//impl<'a> Schema<'a> {
impl Schema {
    pub fn with_config(config: &app::Config) -> Result<Self, sqlx::Error> {
        if config.drop_database {
            print!("removing existing database...");
            std::fs::remove_dir_all(&config.database_url_path)?;
            println!("done.");
        }
    
        match std::fs::create_dir(&config.database_url_path) {
            Ok(()) => Ok(()),
            Err(err) if async_std::io::ErrorKind::AlreadyExists == err.kind() => Ok(()),
            err @ _ => err, 
        }?;
    
        let database_url = format!("{}{}{}", 
            config.database_url_prefix, 
            config.database_url_path, 
            config.database_file);
        
        let connection_options = SqliteConnectOptions::from_str(&database_url)?
            .create_if_missing(true);

        Ok(Self { connection_options })
    }

 /*   pub fn with_pool(sqlite_pool: &'a SqlitePool) -> Schema<'a> {
        Self { sqlite_pool }
    }
*/
/*    pub async fn drop_tables(mut self) -> Schema<'a> {
        sqlx::query("
            DROP TABLE IF EXISTS users;
            DROP TABLE IF EXISTS profiles;
            DROP TABLE IF EXISTS followers;
            DROP TABLE IF EXISTS articles;
            DROP TABLE IF EXISTS favorite_articles;
            DROP TABLE IF EXISTS comments;
        ")
            .execute(self.sqlite_pool)    
            .await
            .expect("Fatal schema failure on dropping tables.");
        self
    }
*/
    pub async fn create(self) -> Result<SqlitePool, sqlx::Error> {
        let sqlite_pool = SqlitePoolOptions::new()
        //        .max_connections(pool_max_connections)
        //        .connect_timeout(pool_timeout)
                .connect_with(self.connection_options)
                .await?;

        sqlx::query("
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL
            );
        ")
        .execute(&sqlite_pool)    
        .await?;
 //       .expect("Fatal schema failure on creating 'user' table.");
        
        sqlx::query("
            CREATE TABLE IF NOT EXISTS profiles (
                user_id INTEGER NOT NULL,
                username TEXT NOT NULL,
                bio TEXT,
                image TEXT,
                FOREIGN KEY (username)
                   REFERENCES users (username) 
                   ON DELETE CASCADE
                   ON UPDATE CASCADE
            );
        ")
        .execute(&sqlite_pool)    
        .await?;
 //       .expect("Fatal schema failure on creating 'profiles' table.");
    
        sqlx::query("
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
        .execute(&sqlite_pool)    
        .await?;
 //       .expect("Fatal schema failure on creating 'followers' table.");
    
        sqlx::query("
            CREATE TABLE IF NOT EXISTS articles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                slug TEXT,
                title TEXT NOT NULL,
                description TEXT,
                body TEXT NOT NULL,
                tagList TEXT,
                createdAt TEXT NOT NULL,
                updatedAt TEXT NOT NULL,
                author TEXT NOT NULL,   
            FOREIGN KEY (author)
                REFERENCES users (username) 
                ON UPDATE CASCADE
            );
        ")
        .execute(&sqlite_pool)    
        .await?;
 //       .expect("Fatal schema failure on creating 'articles' table.");
        
        sqlx::query("
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
        .execute(&sqlite_pool)    
        .await?;
 //       .expect("Fatal schema failure on creating 'favorite_articles' table.");
    
        sqlx::query("
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
        .execute(&sqlite_pool)    
        .await?;
 //       .expect("Fatal schema failure on creating 'comments' table.");

        Ok(sqlite_pool)
    }
}

