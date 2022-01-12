use tide::{Response, Next, Result, StatusCode};
use tide::prelude::*;

use sqlx::prelude::*;
use sqlx::sqlite::{SqlitePool};

use validator::{Validate};
use async_std::fs::File;
use async_std::io::ReadExt;
use crate::{models::{user::*, article::*}, errors::*, db, auth, filters};

pub(crate) type Request = tide::Request<SqlitePool>;

pub(crate) async fn register(mut req: Request) -> tide::Result {
    let wrapped: UserRegWrapped = req.body_json().await?;

    match wrapped.user.validate() {
        Ok(_) => (),
        Err(err) => return FromValidatorError::from(err).into(),
    };

    let res = match db::user::register_user(req.state(), &wrapped.user).await {
        Ok(()) => {
            let mut user = wrapped.user.into();
            auth::Auth::create_token(&mut user)?;
            Ok(json!(UserWrapped { user }).into())
        },
        Err(err) => err.into(),
    };
    res          
}

pub(crate) async fn login(mut req: Request) -> tide::Result {
    println!("in login");
    let login_req: LoginRequestWrapped = req.body_json().await?;
    let res = match db::user::get_user_by_email(req.state(), &login_req.user.email).await {
        Ok(user) => {
            let mut user: User = user.into();
            auth::Auth::create_token(&mut user)?;
            Ok(json!(UserWrapped {user}).into())
        },
        Err(err) => err.into(),
    };
    res          
}

pub(crate) async fn current_user(req: Request) -> tide::Result {

    let (claims, token) = auth::Auth::authorize(&req)?;

    let res = match db::user::get_user_by_username(req.state(), &claims.username).await {
        Ok(user) => {
            let mut user: User = user.into();
            user.token = Some(token);
            Ok(json!(UserWrapped{ user }).into())
        },
        Err(err) => err.into(),
    };
    res
}
              
pub(crate) async fn profile(req: Request) -> tide::Result {
    let username = req.param("username")?;

    let res = match db::user::get_profile(req.state(), &username).await {
        Some(profile) => {
            Ok(json!(ProfileWrapped {profile}).into())
        },
        None => 
        crate::errors::RegistrationError::NoUserFound(username.to_string()).into()
    };
    res          
}


pub(crate) async fn update_user(mut req: Request) -> tide::Result {
    let (claims, token) = auth::Auth::authorize(&req)?;

    let update_user: UserUpdateWrapped = req.body_json().await?;

    db::user::update_user(req.state(), &claims.username, &update_user.user)
        .await
        .and_then(|user| {
            let mut user: User = user.into();
            user.token = Some(token);
            Ok(json!(UserWrapped{ user }).into())
        })
        .or_else(|err| err.into())
}

pub(crate) async fn follow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;

    let (claims, token) = auth::Auth::authorize(&req)?;

    let res = match db::user::follow(req.state(), &claims.username, &celeb_name).await {
        Ok(profile) => {
            let profile: Profile = profile.into();
            Ok(json!(ProfileWrapped{ profile }).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn unfollow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;

    let (claims, token) = auth::Auth::authorize(&req)?;

    let res = match db::user::unfollow(req.state(), &claims.username, &celeb_name).await {
        Ok(profile) => {
            let profile: Profile = profile.into();
            Ok(json!(ProfileWrapped{ profile }).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn create_article(mut req: Request) -> tide::Result {
    let (claims, token) = auth::Auth::authorize(&req)?;

    let article_req: CreateArticleRequestWrapped = req.body_json().await?;

    let res: tide::Result = match db::article::create_article(req.state(), &claims.username, &article_req.article).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped { article }).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn get_article(req: Request) -> tide::Result {
    let slug = req.param("slug")?;
//    let filter = ArticleFilterEnum::BySlug(slug);
    let filter = filters::ArticleFilterBySlug { slug };

    let res = match db::article::get_article(req.state(), filter).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped { article }).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn get_articles(req: Request) -> tide::Result {
//    let filter = ArticleFilterEnum::ByRest(req.query()?);
    let filter: filters::ArticleFilterByValues = req.query()?;
    let order_by = filters::OrderByFilter::Descending("updatedAt");
    let limit_offset: filters::LimitOffsetFilter = req.query()?;

    let res = match db::article::get_articles(req.state(), filter, order_by, limit_offset).await {
        Ok(articles) => {
//            Ok(json!(ArticleResponseWrapped { article }).into())
            Ok(json!(articles).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn update_article(mut req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authorize(&req)?;

    let mut filter: filters::UpdateArticleFilter = req.body_json().await?;
    filter.slug = req.param("slug")?;
    filter.author = &claims.username;

    let res = match db::article::update_article(req.state(), filter).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped { article }).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn favorite_article(req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authorize(&req)?;

    let slug = req.param("slug")?;
    let filter = filters::ArticleFilterBySlug { slug };

    let res = match db::article::favorite_article(req.state(), filter, &claims.username).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped { article }).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn unfavorite_article(req: Request) -> tide::Result {
    let (claims, _) =  auth::Auth::authorize(&req)?;

    let slug = req.param("slug")?;
    let filter = filters::ArticleFilterBySlug { slug };

    let res = match db::article::unfavorite_article(req.state(), filter, &claims.username).await {
        Ok(article) => {
            Ok(json!(ArticleResponseWrapped { article }).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn feed_articles(req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authorize(&req)?;

    let filter = filters::ArticleFilterFeed { follower: &claims.username };
    let order_by = filters::OrderByFilter::Descending("updatedAt");
    let limit_offset: filters::LimitOffsetFilter = req.query()?;

    let res = match db::article::get_articles(req.state(), filter, order_by, limit_offset).await {
        Ok(articles) => {
            Ok(json!(articles).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn add_comment(mut req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authorize(&req)?;

    let wrapped: AddCommentRequestWrapped = req.body_json().await?;
    let slug = req.param("slug")?;
    let author = &claims.username;

    let filter = filters::ArticleFilterBySlug { slug };

    let res = match db::article::add_comment(req.state(), filter, author, &wrapped.comment).await {
        Ok(comment) => {
          Ok(json!(CommentResponseWrapped {comment}).into())
//          Ok(json!("").into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn get_comments(req: Request) -> tide::Result {
    let mut filter = filters::CommentFilterByValues { 
        article_slug: Some(req.param("slug")?),
        author: None
    };

    let tmp = auth::Auth::authorize(&req).ok().and_then(|(claims, _)| Some(claims.username));
    filter.author = tmp.as_deref();

/*        if let Ok((claims, _)) = auth::Auth::authorize(&req) {
        filter.author = Some(&claims.username);
    }*/
    let order_by = filters::OrderByFilter::Descending("id");
    let limit_offset: filters::LimitOffsetFilter = filters::LimitOffsetFilter::default();

    let res = match db::article::get_comments(req.state(), filter, order_by, limit_offset).await {
        Ok(comments) => {
//            Ok(json!(ArticleResponseWrapped { article }).into())
            Ok(json!(comments).into())
        },
        Err(err) => err.into(),
    };
    res
}

pub(crate) async fn get_tags(req: Request) -> tide::Result {

    let res = match db::article::get_tags(req.state()).await {
        Ok(tags) => {
//            Ok(json!(ArticleResponseWrapped { article }).into())
            Ok(json!(tags).into())
        },
        Err(err) => err.into(),
    };
    res
}
