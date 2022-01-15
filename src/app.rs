
use sqlx::sqlite::{SqlitePool};
use crate::endpoints::*;

#[derive(Debug)]
pub(crate) struct Config {
    pub database_url_prefix: String,
    pub database_url_path: String,
    pub database_file: String,
    pub secret: String,
    pub drop_database: bool,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        Self {
            database_url_prefix: std::env::var("DATABASE_URL_PREFIX").expect("No DATABASE_URL_PREFIX environment variable found"),
            database_url_path: std::env::var("DATABASE_URL_PATH").expect("No DATABASE_URL_PATH environment variable found"),
            database_file: std::env::var("DATABASE_FILE").expect("No DATABASE_FILE environment variable found"),
            secret: std::env::var("SECRET").expect("No SECRET environment variable found"),
            drop_database: 0 != std::env::var("DROP_DATABASE")
                .ok()
                .and_then(|s| s.parse::<u32>().ok() )
                .unwrap_or(0) 
        }
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub conn: SqlitePool,
    pub secret: &'static [u8],
}

#[derive(Debug)]
pub(crate) struct App {
    config: Config,
}

impl App {
    pub fn from_config(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(&'static self) -> tide::Result<()> {
        let conn = crate::db::connect(&self.config)
        .await
        .expect("failed to connect to sqlite database. ");

        let mut app = tide::with_state(AppState { conn, secret: self.config.secret.as_bytes()} );

        app.at("/api/users").post(register);
    
        app.at("/api/users/login").post(login);
        app.at("/api/user").get(current_user);
        app.at("/api/user").put(update_user);
        app.at("/api/profiles/:username").get(profile);
        app.at("/api/profiles/:username/follow").post(follow);
        app.at("/api/profiles/:username/follow").delete(unfollow);
        app.at("/api/articles").post(create_article);
        app.at("/api/articles/feed").get(feed_articles);
        app.at("/api/articles").get(get_articles);
        app.at("/api/articles/:slug").put(update_article);
        app.at("/api/articles/:slug").get(get_article);
        app.at("/api/articles/:slug").delete(delete_article);
        app.at("/api/articles/:slug/favorite").post(favorite_article);
        app.at("/api/articles/:slug/favorite").delete(unfavorite_article);
        app.at("/api/articles/:slug/comments").post(add_comment);
        app.at("/api/articles/:slug/comments").get(get_comments);
        app.at("/api/articles/:slug/comments/:id").delete(delete_comment);
        app.at("/api/tags").get(get_tags);
     
        app.listen("127.0.0.1:3000").await?;
    
        Ok(())
    }
}