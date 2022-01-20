
//use sqlx::sqlite::{SqlitePool};
use crate::{config::{Config, HttpConfig}, endpoints::*, backend};



#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub server: backend::Server,
//    pub conn: SqlitePool,
//    pub secret: &'static [u8],
}

#[derive(Debug)]
pub struct App {
//    config: Config,
//    state: Option<AppState>,
}

impl App {
/*    
    pub fn with_config(config: Config) -> Self {
        let state = AppState { server: backend::Server::with_config(config) };
        Self { state: Some(state) }
    } 
*/
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn run(&'static self) -> std::result::Result<(), crate::errors::BackendError> {
 /*       let conn = crate::db::connect(&self.config)
            .await
            .expect("failed to connect to sqlite database. ");
*/
        let cfg = Config::from_env();
        let http_cfg = HttpConfig::from_env();

        let mut state = AppState { server: backend::Server::with_config(cfg) };
        state.server.connect().await?;

        let mut app = tide::with_state(state);

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
     
//        app.listen("127.0.0.1:3000").await?;
        let hp = format!("{}:{}", http_cfg.host, http_cfg.http_port); 
        app.listen(hp).await?;
    
        Ok(())
    }
}