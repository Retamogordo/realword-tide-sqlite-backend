mod db;
mod auth;
mod errors;
mod filters;
mod models;
mod utils;
mod endpoints;
mod app;

use async_std::fs::File;
use async_std::io::ReadExt;
use crate::endpoints::*;
use once_cell::sync::OnceCell;

static APP: OnceCell<app::App> = OnceCell::new();

#[async_std::main]
async fn main() -> tide::Result<()> {
    let cfg = app::Config::from_env();
    let app = app::App::from_config(cfg);

    APP.set(app).expect("Cannot create application instance.");
    
    APP.get().unwrap().run().await
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

