use sqlx::sqlite::{SqlitePool};

pub(crate) struct Schema<'a> {
    sqlite_pool: &'a SqlitePool,
}

impl<'a> Schema<'a> {
    pub fn with_pool(sqlite_pool: &'a SqlitePool) -> Schema<'a> {
        Self { sqlite_pool }
    }

    pub async fn drop_tables(mut self) -> Schema<'a> {
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

    pub async fn create(mut self) -> Schema<'a> {
        sqlx::query("
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL
            );
        ")
        .execute(self.sqlite_pool)    
        .await
        .expect("Fatal schema failure on creating 'user' table.");
        
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
        .execute(self.sqlite_pool)    
        .await
        .expect("Fatal schema failure on creating 'profiles' table.");
    
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
        .execute(self.sqlite_pool)    
        .await
        .expect("Fatal schema failure on creating 'followers' table.");
    
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
        .execute(self.sqlite_pool)    
        .await
        .expect("Fatal schema failure on creating 'articles' table.");
        
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
        .execute(self.sqlite_pool)    
        .await
        .expect("Fatal schema failure on creating 'favorite_articles' table.");
    
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
        .execute(self.sqlite_pool)    
        .await
        .expect("Fatal schema failure on creating 'comments' table.");

        self
    }
}

