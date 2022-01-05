use tide::{Response, Next, Result, StatusCode};
use tide::prelude::*;
use sqlx::sqlite::{SqlitePool};
use std::future::Future;
use std::pin::Pin;
use validator::{Validate};
use async_std::fs::File;
use async_std::io::ReadExt;
//use once_cell::sync::OnceCell;
//use async_trait::async_trait;

mod db;
mod auth;
mod errors;

pub(crate) type Request = tide::Request<SqlitePool>;

#[derive(Debug, Deserialize)]
struct Author {
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserReg {
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserRegWrapped {
//    user: String,
    user: UserReg,
}

#[derive(Debug, Deserialize)]
struct UserUpdate {
    email: String,
    bio: String,
    image: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserWrapped {
    user: User,
}

impl UserWrapped {
    fn from_user(user: User) -> Self {
        Self { user }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(sqlx::FromRow)]
struct User {
    email: String,    
    token: Option<String>,    
    username: String,    
    bio: String,    
    image: Option<String>,  
}

impl From<UserReg> for User {
    fn from(user_reg: UserReg) -> Self {
        Self {
            username: user_reg.username,
            email: user_reg.email,
            token: None,
            bio: "".to_string(), 
            image: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(sqlx::FromRow)]
struct Profile {
    username: String,    
    bio: String,    
    image: Option<String>,  
}

#[derive(Debug, Serialize, Deserialize)]
struct ProfileWrapped {
    profile: Profile,
}

impl ProfileWrapped {
    fn from_profile(profile: Profile) -> Self {
        Self { profile }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct LoginRequestWrapped {
//    user: String,
    user: LoginRequest,
}

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
//    app.at("/api/user").put(update_user);
    app.at("/api/profiles/:username").get(profile);
//    app.at("/api/profiles/*/follow").post(follow);
//    app.at("/api/profiles/*/follow").delete(unfollow);
//    app.at("/api/articles").get(articles);
 
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

async fn register(mut req: Request) -> tide::Result {
//    let res = req.body_string().await;
    println!("in register");
//    tide::Response
    let user: UserRegWrapped = req.body_json().await?;

    user.user.validate()?;

//    let user = UserRegWrapped { username: "dummy".to_string(), email: "qqq".to_string(), password: "ss".to_string() };
    let res = match db::register_user(req.state(), &user.user).await {
        Ok(()) => {
            let mut user = user.user.into();
            auth::Auth::create_token(&mut user)?;
            Ok(json!(UserWrapped::from_user(user)).into())
        },
        Err(err) => Ok(err.into()),
    };
    res          
}

async fn login(mut req: Request) -> tide::Result {
    println!("in login");
    let login_req: LoginRequestWrapped = req.body_json().await?;
    let res = match db::get_user(req.state(), &login_req.user.email).await {
        Ok(user) => {
            let mut user: User = user.into();
            auth::Auth::create_token(&mut user)?;
            Ok(json!(UserWrapped::from_user(user)).into())
        },
        Err(err) => Ok(err.into()),
    };
    res          
}

async fn current_user(req: Request) -> tide::Result {

    let (claims, token) = match auth::Auth::authorize(&req) {
        Ok(claims) => claims,
        Err(err) => return Ok(err.into()),
    };

    let res = match db::get_user(req.state(), &claims.email).await {
        Ok(user) => {
            let mut user: User = user.into();
            user.token = Some(token);
            Ok(json!(UserWrapped::from_user(user)).into())
        },
        Err(err) => Ok(err.into()),
    };
    res
}
              
async fn profile(req: Request) -> tide::Result {
    let username = req.param("username")?;

    let res = match db::get_profile(req.state(), &username).await {
        Ok(profile) => {
            Ok(json!(ProfileWrapped::from_profile(profile)).into())
        },
        Err(err) => Ok(err.into()),
    };
    res          
}


    async fn update_user(mut req: Request) -> tide::Result {
        let update: UserUpdate = req.body_json().await?;
        
        let user = json! ({
            "user": {    
                "email": update.email,    
                "token": "jwt.token.here",    
                "username": "jake",    
                "bio": update.bio,    
                "image": update.image  
            }
        });
    
        Ok(user.into())
    }

    async fn follow(mut req: Request) -> tide::Result {
        let profile = json! ({
            "profile": {
                "username": "jake",
                "bio": "I work at statefarm",
                "image": "https://api.realworld.io/images/smiley-cyrus.jpg",
                "following": false
            }
        });
        Ok(profile.into())
    }

    async fn unfollow(mut req: Request) -> tide::Result {
        let profile = json! ({
            "profile": {
                "username": "jake",
                "bio": "I work at statefarm",
                "image": "https://api.realworld.io/images/smiley-cyrus.jpg",
                "following": false
            }
        });
        Ok(profile.into())
    }
    async fn articles(mut req: Request) -> tide::Result {
        let author: Author = req.body_json().await?;

        let articles = json! ({
                "articles":[{
                  "slug": "how-to-train-your-dragon",
                  "title": "How to train your dragon",
                  "description": "Ever wonder how?",
                  "body": "It takes a Jacobian",
                  "tagList": ["dragons", "training"],
                  "createdAt": "2016-02-18T03:22:56.637Z",
                  "updatedAt": "2016-02-18T03:48:35.824Z",
                  "favorited": false,
                  "favoritesCount": 0,
                  "author": {
                    "username": "jake",
                    "bio": "I work at statefarm",
                    "image": "https://i.stack.imgur.com/xHWG8.jpg",
                    "following": false
                  }
                }, {
                  "slug": "how-to-train-your-dragon-2",
                  "title": "How to train your dragon 2",
                  "description": "So toothless",
                  "body": "It a dragon",
                  "tagList": ["dragons", "training"],
                  "createdAt": "2016-02-18T03:22:56.637Z",
                  "updatedAt": "2016-02-18T03:48:35.824Z",
                  "favorited": false,
                  "favoritesCount": 0,
                  "author": {
                    "username": "jake",
                    "bio": "I work at statefarm",
                    "image": "https://i.stack.imgur.com/xHWG8.jpg",
                    "following": false
                  }
                }],
                "articlesCount": 2     
        });
        Ok(articles.into())
    }

