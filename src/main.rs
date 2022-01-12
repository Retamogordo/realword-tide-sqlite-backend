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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
struct CreateArticleRequest { 
    #[serde(skip_deserializing)]
    slug: String,
    title: String,
    description: Option<String>,
    body: String,
    tag_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
struct CreateArticleRequestWrapped { 
    #[serde(deserialize_with = "slugify_article_on_create")]    
    article: CreateArticleRequest,
}

fn slugify_article_on_create<'de, D>(deserializer: D) -> std::result::Result<CreateArticleRequest, D::Error>
where
    D: serde::Deserializer<'de>, {
    use slugify::slugify;
    let mut req: CreateArticleRequest = serde::Deserialize::deserialize(deserializer)?;
    req.slug = slugify!(&req.title);
    Ok(req)
}

#[derive(sqlx::FromRow)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub(crate) struct Comment { 
    id: i32,
    #[serde(serialize_with = "transform_datetime")]    
    created_at: chrono::DateTime<chrono::Utc>,
    #[serde(serialize_with = "transform_datetime")]    
    updated_at: chrono::DateTime<chrono::Utc>,
    body: String,
    #[serde(skip_serializing)]
    author: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CommentResponse {
    pub author: Option<Profile>,
    #[serde(flatten)]
    pub comment: Comment,
}

#[derive(Debug, Serialize)]
struct CommentResponseWrapped {
    comment: CommentResponse,
}

impl CommentResponseWrapped {
    fn from_comment(comment: CommentResponse) -> Self {
        Self { comment }
    }
}


#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MultipleCommentResponse {
    pub comments: Vec<CommentResponse>,
}

impl MultipleCommentResponse {
    fn from_comments(comments: Vec<CommentResponse>) -> Self {
        Self { 
            comments,
        }
    }
}


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
struct AddCommentRequest { 
    body: String,
}

#[derive(Debug, Deserialize, Clone)]
struct AddCommentRequestWrapped { 
    comment: AddCommentRequest,
}

#[derive(sqlx::FromRow)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub(crate) struct Article { 
//    #[serde(skip_serializing)]
    slug: Option<String>,
    title: String,
    description: Option<String>,
    body: String,
    #[serde(serialize_with = "transform_string_to_vec")]    
    tag_list: Option<String>,
    #[serde(serialize_with = "transform_datetime")]    
    created_at: chrono::DateTime<chrono::Utc>,
    #[serde(serialize_with = "transform_datetime")]    
    updated_at: chrono::DateTime<chrono::Utc>,
    favorited: bool,
    favorites_count: u32,
    #[serde(skip_serializing)]
    author: String,
}

fn transform_string_to_vec<S>(tag_list: &Option<String>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer, {

    tag_list.as_ref().and_then(|tags| {
        let mut tag_vec = tags.split(",")
            .map(|tag| tag.trim())
            .filter(|tag| *tag != "")
            .collect::<Vec<&str>>();
            
        tag_vec.sort_by_key(|tag| tag.to_lowercase());
        Some(tag_vec)
    })
    .serialize(serializer)
}

fn transform_datetime<S>(dt: &chrono::DateTime<chrono::Utc>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer, {
        dt.checked_add_signed(chrono::Duration::milliseconds(42))
        .serialize(serializer)
}

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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TagList {
    pub tags: Vec<String>,
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

async fn register(mut req: Request) -> tide::Result {
//    let res = req.body_string().await;
    println!("in register");
//    tide::Response
    let mut wrapped: UserRegWrapped = req.body_json().await?;

//    user.user.email = "asd".to_string();

    match wrapped.user.validate() {
        Ok(_) => (),
        Err(err) => return Ok(errors::FromValidatorError(err).into())
    };
/*
    Ok(json!(UserWrapped::from_user(
        db::register_user(req.state(), &user.user)
        .await?
//        .or_else(|err| err.into())?
    )).into())
*/
    let res = match db::register_user(req.state(), &wrapped.user).await {
        Ok(()) => {
            let mut user = wrapped.user.into();
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

    let (claims, token) = auth::Auth::authorize(&req)?;

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
    let (claims, token) = auth::Auth::authorize(&req)?;

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

    let (claims, token) = auth::Auth::authorize(&req)?;

    let res = match db::follow(req.state(), &claims.username, &celeb_name).await {
        Ok(profile) => {
            let profile: Profile = profile.into();
            Ok(json!(ProfileWrapped::from_profile(profile)).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn unfollow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;

    let (claims, token) = auth::Auth::authorize(&req)?;

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
    let (claims, token) = auth::Auth::authorize(&req)?;

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

pub(crate) enum OrderByFilter<'a> {
    Ascending(&'a str),
    Descending(&'a str),
    None,
}

impl std::fmt::Display for OrderByFilter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {         
        match self {
            OrderByFilter::Ascending(row_name) => write!( f, "ORDER BY {} ASC ", row_name),
            OrderByFilter::Descending(row_name) => write!( f, "ORDER BY {} DESC ", row_name),
            OrderByFilter::None => Ok(()),
        }
    }
}

pub(crate) struct CommentFilterByValues<'a> {
    pub author: Option<&'a str>,
    pub article_slug: Option<&'a str>,
//    pub order: OrderByFilter<'a>,
}

impl Default for CommentFilterByValues<'_> {
    fn default() -> Self {
        Self { 
            author: None,
            article_slug: None,
        }
    }
}

impl std::fmt::Display for CommentFilterByValues<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.author.as_ref().map(|val| write!( f, " {}='{}' AND", "author", val) ).unwrap_or(Ok(()))?;
        self.article_slug.as_ref().map(|val|
            write!( f, 
                " {} IN (SELECT id FROM articles WHERE slug='{}') AND", 
                "article_id", val)
        ).unwrap_or(Ok(()))?;

        write!( f, " 1=1 ")
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub(crate) struct UpdateArticleRequest { 
    title: Option<String>,
    description: Option<String>,
    body: Option<String>,
    #[serde(skip_deserializing)]
    pub slug_from_title: Option<String>,
}

fn slugify_article_on_update<'de, D>(deserializer: D) -> std::result::Result<UpdateArticleRequest, D::Error>
where
    D: serde::Deserializer<'de>, {
    use slugify::slugify;
    let mut req: UpdateArticleRequest = serde::Deserialize::deserialize(deserializer)?;
    req.slug_from_title = req.title.as_ref().and_then(|title| Some(slugify!(title)));
    Ok(req)
}

#[derive(Debug, Deserialize, Clone)]
//#[serde(default)]
pub(crate) struct UpdateArticleFilter<'a> {
    #[serde(skip_deserializing)]
    pub slug: &'a str,
    #[serde(skip_deserializing)]
    pub author: &'a str,
    #[serde(deserialize_with = "slugify_article_on_update")]    
    article: UpdateArticleRequest,
}

impl UpdateArticleFilter<'_> {
    pub fn updated_slug(&self) -> &str {
        if let Some(ref slug) = self.article.slug_from_title {
            slug
        } else { 
            self.slug
        }
    }
}
/*
impl Default for UpdateArticleFilter<'_> {
    fn default() -> Self {
        Self { 
            slug: "",
            title: None,
            description: None,
            body: None, 
        }
    }
}
*/
impl std::fmt::Display for UpdateArticleFilter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use slugify::slugify;

        self.article.title.as_ref().map(|val| 
            write!( f, " {}='{}' , {}='{}'", "title", val, "slug", slugify!(val)) ).unwrap_or(Ok(()))?;
        self.article.description.as_ref().map(|val| write!( f, " {}='{}' ,", "description", val) ).unwrap_or(Ok(()))?;
        self.article.body.as_ref().map(|val| write!( f, " {}='{}' ,", "body", val) ).unwrap_or(Ok(()))?;
        write!( f, " id=id ")?;
        write!( f, " WHERE slug='{}' AND author='{}'", self.slug, self.author)
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
    let order_by = OrderByFilter::Descending("updatedAt");
    let limit_offset: LimitOffsetFilter = req.query()?;

    let res = match db::get_articles(req.state(), filter, order_by, limit_offset).await {
        Ok(articles) => {
//            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
            Ok(json!(articles).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn update_article(mut req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authorize(&req)?;

    let mut filter: UpdateArticleFilter = req.body_json().await?;
    filter.slug = req.param("slug")?;
    filter.author = &claims.username;

    let res = match db::update_article(req.state(), filter).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn favorite_article(req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authorize(&req)?;

    let slug = req.param("slug")?;
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
    let (claims, _) =  auth::Auth::authorize(&req)?;

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
    let (claims, _) = auth::Auth::authorize(&req)?;

    let filter = ArticleFilterFeed { follower: &claims.username };
    let order_by = OrderByFilter::Descending("updatedAt");
    let limit_offset: LimitOffsetFilter = req.query()?;

    let res = match db::get_articles(req.state(), filter, order_by, limit_offset).await {
        Ok(articles) => {
            Ok(json!(articles).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn add_comment(mut req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authorize(&req)?;

    let wrapped: AddCommentRequestWrapped = req.body_json().await?;
    let slug = req.param("slug")?;
    let author = &claims.username;

    let filter = ArticleFilterBySlug { slug };

    let res = match db::add_comment(req.state(), filter, author, &wrapped.comment).await {
        Ok(comment) => {
          Ok(json!(CommentResponseWrapped::from_comment(comment)).into())
//          Ok(json!("").into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn get_comments(req: Request) -> tide::Result {
    let mut filter = CommentFilterByValues { 
        article_slug: Some(req.param("slug")?),
        author: None
    };

    let tmp = auth::Auth::authorize(&req).ok().and_then(|(claims, _)| Some(claims.username));
    filter.author = tmp.as_deref();

/*        if let Ok((claims, _)) = auth::Auth::authorize(&req) {
        filter.author = Some(&claims.username);
    }*/
    let order_by = OrderByFilter::Descending("id");
    let limit_offset: LimitOffsetFilter = LimitOffsetFilter::default();

    let res = match db::get_comments(req.state(), filter, order_by, limit_offset).await {
        Ok(comments) => {
//            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
            Ok(json!(comments).into())
        },
        Err(err) => err.into(),
    };
    res
}

async fn get_tags(req: Request) -> tide::Result {

    let res = match db::get_tags(req.state()).await {
        Ok(tags) => {
//            Ok(json!(ArticleResponseWrapped::from_article(article)).into())
            Ok(json!(tags).into())
        },
        Err(err) => err.into(),
    };
    res
}
