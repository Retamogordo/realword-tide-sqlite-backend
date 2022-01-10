use tide::{Response, Next, Result, StatusCode};
use tide::prelude::*;

use sqlx::prelude::*;
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
pub(crate) struct UserReg {
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserUpdateWrapped {
//    user: String,
    user: UserUpdate,
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
    following: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
struct CreateArticleRequest { 
    #[serde(skip_deserializing)]
    slug: String,
    title: String,
    description: Option<String>,
    body: String,
    tag_list: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CreateArticleRequestWrapped { 
    #[serde(deserialize_with = "slugify_article")]    
    article: CreateArticleRequest,
}

fn slugify_article<'de, D>(deserializer: D) -> std::result::Result<CreateArticleRequest, D::Error>
where
    D: serde::Deserializer<'de>, {
    use slugify::slugify;
    let mut req: CreateArticleRequest = serde::Deserialize::deserialize(deserializer)?;
    req.slug = slugify!(&req.title);
    Ok(req)
}

#[derive(sqlx::FromRow)]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub(crate) struct Article { 
    #[serde(skip_serializing)]
    slug: Option<String>,
    title: String,
    description: Option<String>,
    body: String,
    tag_list: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    favorited: bool,
    favorites_count: u32,
    #[serde(skip_serializing)]
    author: String,
}

use serde::ser::SerializeStruct;
impl Serialize for Article {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Article", 9)?;
        state.serialize_field("slug", &self.slug)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("body", &self.body)?;
        state.serialize_field("tagList", 
            &self.tag_list.as_ref().and_then(|tags| 
                Some(tags.split(",")
                    .filter(|tag| *tag != "")
                    .map(|tag| tag.to_string())
                    .collect::<Vec<String>>()))
        )?;
        // quick and dirty - needed to add some dummy millis to fit to conduit deser format
        state.serialize_field("createdAt", &self.created_at.checked_add_signed(chrono::Duration::milliseconds(42)))?;
        state.serialize_field("updatedAt", &self.updated_at.checked_add_signed(chrono::Duration::milliseconds(42)))?;
        state.serialize_field("favorited", &self.favorited)?;
        state.serialize_field("favoritesCount", &self.favorites_count)?;
        state.end()
    }
}

//#[derive(Debug, Serialize, Deserialize, Clone)]
//pub(crate) struct AuthorWrapped(Option<Profile>)

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ArticleResponse {
    pub author: Option<Profile>,
    #[serde(flatten)]
    pub article: Article,
}


#[derive(Debug, Serialize)]
struct ArticleResponseWrapped {
    article: ArticleResponse,
}

impl ArticleResponseWrapped {
    fn from_article(article: ArticleResponse) -> Self {
        Self { article }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MultipleArticleResponse {
    pub articles: Vec<ArticleResponse>,
    articles_count: usize,
}

impl MultipleArticleResponse {
    fn from_articles(articles: Vec<ArticleResponse>) -> Self {
        let articles_count = articles.len();
        Self { 
            articles,
            articles_count
        }
    }
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
    app.at("/api/user").put(update_user);
    app.at("/api/profiles/:username").get(profile);
    app.at("/api/profiles/:username/follow").post(follow);
    app.at("/api/profiles/:username/follow").delete(unfollow);
    app.at("/api/articles").post(create_article);
    app.at("/api/articles/feed").get(feed_articles);
    app.at("/api/articles").get(get_articles);
    app.at("/api/articles/:slug").get(get_article);
    app.at("/api/articles/:slug/favorite").post(favorite_article);
    app.at("/api/articles/:slug/favorite").delete(unfavorite_article);
 
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
    let mut user: UserRegWrapped = req.body_json().await?;

//    user.user.email = "asd".to_string();

    match user.user.validate() {
        Ok(_) => (),
        Err(err) => return Ok(errors::FromValidatorError(err).into())
    };

    let res = match db::register_user(req.state(), &user.user).await {
        Ok(()) => {
            let mut user = user.user.into();
            auth::Auth::create_token(&mut user)?;
            Ok(json!(UserWrapped::from_user(user)).into())
        },
        Err(err) => err.into(),
    };
    res          
}

async fn login(mut req: Request) -> tide::Result {
    println!("in login");
    let login_req: LoginRequestWrapped = req.body_json().await?;
    let res = match db::get_user_by_email(req.state(), &login_req.user.email).await {
        Ok(user) => {
            let mut user: User = user.into();
            auth::Auth::create_token(&mut user)?;
            Ok(json!(UserWrapped::from_user(user)).into())
        },
        Err(err) => err.into(),
    };
    res          
}

async fn current_user(req: Request) -> tide::Result {

    let (claims, token) = match auth::Auth::authorize(&req) {
        Ok(claims) => claims,
        Err(err) => return Ok(err.into()),
    };

    let res = match db::get_user_by_username(req.state(), &claims.username).await {
        Ok(user) => {
            let mut user: User = user.into();
            user.token = Some(token);
            Ok(json!(UserWrapped::from_user(user)).into())
        },
        Err(err) => err.into(),
    };
    res
}
              
async fn profile(req: Request) -> tide::Result {
    let username = req.param("username")?;

    let res = match db::get_profile(req.state(), &username).await {
        Some(profile) => {
            Ok(json!(ProfileWrapped::from_profile(profile)).into())
        },
        None => 
        crate::errors::RegistrationError::NoUserFound(username.to_string()).into()
    };
    res          
}


async fn update_user(mut req: Request) -> tide::Result {
    let (claims, token) = match auth::Auth::authorize(&req) {
        Ok(claims) => claims,
        Err(err) => return Ok(err.into()),
    };

    let update_user: UserUpdateWrapped = req.body_json().await?;

    let res = match db::update_user(req.state(), &claims.username, &update_user.user).await {
        Ok(user) => {
            let mut user: User = user.into();
            user.token = Some(token);
            Ok(json!(UserWrapped::from_user(user)).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn follow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;

    let (claims, token) = match auth::Auth::authorize(&req) {
        Ok(claims) => claims,
        Err(err) => return Ok(err.into()),
    };

    let res = match db::follow(req.state(), &claims.username, &celeb_name).await {
        Ok(profile) => {
            let mut profile: Profile = profile.into();
            Ok(json!(ProfileWrapped::from_profile(profile)).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn unfollow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;

    let (claims, _) = match auth::Auth::authorize(&req) {
        Ok(claims) => claims,
        Err(err) => return Ok(err.into()),
    };

    let res = match db::unfollow(req.state(), &claims.username, &celeb_name).await {
        Ok(profile) => {
            let profile: Profile = profile.into();
            Ok(json!(ProfileWrapped::from_profile(profile)).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn create_article(mut req: Request) -> tide::Result {
    let (claims, _) = match auth::Auth::authorize(&req) {
        Ok(claims) => claims,
        Err(err) => return Ok(err.into()),
    };

    let article_req: CreateArticleRequestWrapped = req.body_json().await?;

    let res = match db::create_article(req.state(), &claims.username, &article_req.article).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
        },
        Err(err) => err.into(),
    };
    res
}

trait ArticleFilter: std::fmt::Display {}
/*
pub(crate) enum ArticleFilterEnum<'sl> {
    BySlug(&'sl str),
    ByRest(ArticleFilterByValues),
}

impl<'sl> ArticleFilterEnum<'sl> {
    fn from_slug(slug: &'sl str) -> Self {
        Self::BySlug(slug)
    }
}

impl std::fmt::Display for ArticleFilterEnum<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BySlug(slug) => write!( f, " {}='{}'", "slug", slug),
            Self::ByRest(filter) => filter.fmt(f),        
        }
    }
}*/

#[derive(Deserialize)]
pub(crate) struct ArticleFilterBySlug<'a> {
    pub slug: &'a str,
}

impl ArticleFilter for ArticleFilterBySlug<'_> {}
impl std::fmt::Display for ArticleFilterBySlug<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!( f, " {}='{}'", "slug", self.slug)
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub(crate) struct ArticleFilterByValues {
    pub author: Option<String>,
    pub tag: Option<String>,
    pub favorited: Option<String>,
}

impl ArticleFilter for ArticleFilterByValues {}

impl Default for ArticleFilterByValues {
    fn default() -> Self {
        Self { 
            author: None,
            tag: None,
            favorited: None,
        }
    }
}

impl std::fmt::Display for ArticleFilterByValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.author.as_ref().map(|val| write!( f, " {}='{}' AND", "author", val) ).unwrap_or(Ok(()))?;
        self.tag.as_ref().map(|val| write!( f, " {} LIKE '%{}%' AND", "tagList", val) ).unwrap_or(Ok(()))?;
        self.favorited.as_ref().map(|val| 
            write!( f, " {}='{}' AND", "favorite_articles.username", val) 
        ).unwrap_or(Ok(()))?;
//        self.slug.as_ref().map(|val| write!( f, " {}='{}' AND", "slug", val) ).unwrap_or(Ok(()))?;

        //        write!( f, " {}={} AND", "favorited", if self.favorited {1} else {0})?;
        write!( f, " 1=1")
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub(crate) struct ArticleFilterFeed<'a> {
    pub follower: &'a str,
}

impl ArticleFilter for ArticleFilterFeed<'_> {}

impl std::fmt::Display for ArticleFilterFeed<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!( f, 
            " {} IN (SELECT celeb_name FROM followers WHERE follower_name='{}')", 
            "author", self.follower)
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub(crate) struct LimitOffsetFilter {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl Default for LimitOffsetFilter {
    fn default() -> Self {
        Self { 
            limit: None,
            offset: None,
        }
    }
}

impl std::fmt::Display for LimitOffsetFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.limit.as_ref().map(|val| write!( f, "LIMIT {} ", val) ).unwrap_or(Ok(()))?;
        self.offset.as_ref().map(|val| write!( f, "OFFSET {}", val) ).unwrap_or(Ok(()))
    }
}

async fn get_article(req: Request) -> tide::Result {
    let slug = req.param("slug")?;
//    let filter = ArticleFilterEnum::BySlug(slug);
    let filter = ArticleFilterBySlug { slug };

    let res = match db::get_article(req.state(), filter).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn get_articles(req: Request) -> tide::Result {
//    let filter = ArticleFilterEnum::ByRest(req.query()?);
    let filter: ArticleFilterByValues = req.query()?;
    let limit_offset: LimitOffsetFilter = req.query()?;

    let res = match db::get_articles(req.state(), filter, limit_offset).await {
        Ok(articles) => {
//            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
            Ok(json!(articles).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn favorite_article(req: Request) -> tide::Result {
    let (claims, _) = match auth::Auth::authorize(&req) {
        Ok(claims) => claims,
        Err(err) => return Ok(err.into()),
    };

    let slug = req.param("slug")?;
//    let filter = ArticleFilterEnum::BySlug(slug);
    let filter = ArticleFilterBySlug { slug };

    let res = match db::favorite_article(req.state(), filter, &claims.username).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn unfavorite_article(req: Request) -> tide::Result {
    let (claims, _) = match auth::Auth::authorize(&req) {
        Ok(claims) => claims,
        Err(err) => return Ok(err.into()),
    };

    let slug = req.param("slug")?;
    let filter = ArticleFilterBySlug { slug };

    let res = match db::unfavorite_article(req.state(), filter, &claims.username).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn feed_articles(req: Request) -> tide::Result {
    let (claims, _) = match auth::Auth::authorize(&req) {
        Ok(claims) => claims,
        Err(err) => return Ok(err.into()),
    };

    let filter = ArticleFilterFeed { follower: &claims.username };
    let limit_offset: LimitOffsetFilter = req.query()?;

    let res = match db::get_articles(req.state(), filter, limit_offset).await {
        Ok(articles) => {
//            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
            Ok(json!(articles).into())
        },
        Err(err) => err.into(),
    };
    res
}

