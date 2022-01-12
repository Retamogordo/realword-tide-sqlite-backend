mod db;
mod auth;
mod errors;
mod filters;
mod models;
mod utils;
mod endpoints;

use tide::{Response, Next, Result, StatusCode};
use tide::prelude::*;

use sqlx::prelude::*;
use sqlx::sqlite::{SqlitePool};

//use std::future::Future;
//use std::pin::Pin;
use validator::{Validate};
use async_std::fs::File;
use async_std::io::ReadExt;
use crate::endpoints::*;
//use once_cell::sync::OnceCell;
//use async_trait::async_trait;


#[derive(Clone)]
struct MyState {
    name: String,
}

struct MyMiddle {
//    conn: &'static sqlx::Pool<sqlx::Sqlite>,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    dotenv::dotenv().ok();
    
    let database_url_prefix =
        std::env::var("DATABASE_URL_PREFIX").expect("No DATABASE_URL_PREFIX environment variable found");
    let database_url_path =
        std::env::var("DATABASE_URL_PATH").expect("No DATABASE_URL_PATH environment variable found");
    let database_file =
        std::env::var("DATABASE_FILE").expect("No DATABASE_FILE environment variable found");

    let conn_db = crate::db::connect(&database_url_prefix, &database_url_path, &database_file).await?;
{
//    let mut app = tide::with_state(conn_db);
    let mut app = tide::with_state(conn_db);
//    let mut app = tide::new();
//    app.at("/").get(index);
    app.at("/api/users").post(register);
//    app.at("/login_register").get(login_register);
//    app.with(MyMiddle  {}).at("/api/users");

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
    app.at("/api/articles/:slug/favorite").post(favorite_article);
    app.at("/api/articles/:slug/favorite").delete(unfavorite_article);
    app.at("/api/articles/:slug/comments").post(add_comment);
    app.at("/api/articles/:slug/comments").get(get_comments);
    app.at("/api/tags").get(get_tags);
 
    app.listen("127.0.0.1:3000").await?;
}

    Ok(())
}

async fn login_register(_req: Request) -> tide::Result {
    let mut hdr_file = File::open("./src/templates/header.html").await?;
    let mut home_file = File::open("./src/templates/login_register.html").await?;
    let mut ft_file = File::open("./src/templates/footer.html").await?;

    let mut hdr_contents = Vec::new();
    hdr_file.read_to_end(&mut hdr_contents).await?;
    let mut home_contents = Vec::new();
    home_file.read_to_end(&mut home_contents).await?;
    let mut ft_contents = Vec::new();
    ft_file.read_to_end(&mut ft_contents).await?;

    let mut index_contents = hdr_contents;
    index_contents.extend(home_contents);
    index_contents.extend(ft_contents);

    let body = tide::Body::from_bytes(index_contents);

    let response = tide::Response::builder(200)
        .body(body)
        .content_type(http_types::mime::HTML)
        .build();
    Ok(response)
}

async fn index(_req: Request) -> tide::Result {
    //    let res = req.body_string().await;
    let mut hdr_file = File::open("./src/templates/header.html").await?;
    let mut home_file = File::open("./src/templates/home.html").await?;
    let mut ft_file = File::open("./src/templates/footer.html").await?;

    let mut hdr_contents = Vec::new();
    hdr_file.read_to_end(&mut hdr_contents).await?;
    let mut home_contents = Vec::new();
    home_file.read_to_end(&mut home_contents).await?;
    let mut ft_contents = Vec::new();
    ft_file.read_to_end(&mut ft_contents).await?;

    let mut index_contents = hdr_contents;
    index_contents.extend(home_contents);
    index_contents.extend(ft_contents);

 //   let body = tide::Body::from_file("./index.html").await?;
    let body = tide::Body::from_bytes(index_contents);

    let response = tide::Response::builder(200)
        .body(body)
        .content_type(http_types::mime::HTML)
        .build();

        Ok(response)
}

async fn src(_req: Request) -> tide::Result {
    //    let res = req.body_string().await;
        println!("in src");

        let body = tide::Body::from_file("./src/index.html").await?;

        let response = tide::Response::builder(200)
            .body(body)
            .content_type(http_types::mime::HTML)
            .build();

    
    //    let user = UserRegWrapped { username: "dummy".to_string(), email: "qqq".to_string(), password: "ss".to_string() };
        Ok(response)
}

