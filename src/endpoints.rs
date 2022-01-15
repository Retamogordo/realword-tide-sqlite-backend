use tide::prelude::*;

use validator::{Validate};
//use async_std::fs::File;
//use async_std::io::ReadExt;
use crate::app::AppState;
use crate::{models::{user::*, article::*}, errors::*, db, auth, filters};

pub(crate) type Request = tide::Request<AppState>;

pub(crate) async fn register(mut req: Request) -> tide::Result {
    let wrapped: UserRegWrapped = req.body_json().await?;

    match wrapped.user.validate() {
        Ok(_) => (),
        Err(err) => return FromValidatorError::from(err).into(),
    };

    let res = match db::user::register_user(&req.state().conn, &wrapped.user).await {
        Ok(()) => {
            let mut user: User = wrapped.user.into();
//            auth::Auth::create_token(&mut user)?;
            user.token = Some(auth::Auth::create_token(&user, &req.state().secret)?);
            Ok(json!(user.wrap()).into())
        },
        Err(err) => err.into(),
    };
    res          
}

pub(crate) async fn login(mut req: Request) -> tide::Result {
    println!("in login");
    let login_req: LoginRequestWrapped = req.body_json().await?;
    let filter = filters::UserFilter::default().email(&login_req.user.email);

    let res = match db::user::get_user(&req.state().conn, filter).await {
        Ok(user) => {
            let mut user: User = user.into();
            user.token = Some(auth::Auth::create_token(&user, &req.state().secret)?);
            Ok(json!(user.wrap()).into())
        },
        Err(err) => err.into(),
    };
    res          
}

pub(crate) async fn current_user(req: Request) -> tide::Result {

    let (claims, token) = auth::Auth::authenticate(&req)?;
    let filter = filters::UserFilter::default().username(&claims.username);
    
    let res = match db::user::get_user(&req.state().conn, filter).await {
        Ok(user) => {
            let mut user: User = user.into();
            user.token = Some(token);
            Ok(json!(user.wrap()).into())
        },
        Err(err) => err.into(),
    };
    res
}
              
pub(crate) async fn profile(req: Request) -> tide::Result {
    let username = req.param("username")?;

    let res = match db::user::get_profile(&req.state().conn, &username).await {
        Some(profile) => {
            Ok(json!(profile.wrap()).into())
        },
        None => 
        crate::errors::BackendError::NoUserFound(username.to_string()).into()
    };
    res          
}

pub(crate) async fn update_user(mut req: Request) -> tide::Result {
    let (claims, token) = auth::Auth::authenticate(&req)?;

    let update_user: UserUpdateWrapped = req.body_json().await?;
    let filter = filters::UpdateUserFilter::default().username(&claims.username);

    db::user::update_user(&req.state().conn, &update_user.user, filter)
        .await
        .and_then(|user| {
            let mut user: User = user.into();
            user.token = Some(token);
            Ok(json!(user.wrap()).into())
        })
        .or_else(|err| err.into())
}

pub(crate) async fn follow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;

    let (claims, _) = auth::Auth::authenticate(&req)?;

    db::user::follow(&req.state().conn, &claims.username, &celeb_name)
        .await
        .and_then(|profile| {
            let profile: Profile = profile.into();
            Ok(json!(profile.wrap()).into())
        })
        .or_else(|err| err.into())
}

pub(crate) async fn unfollow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;

    let (claims, token) = auth::Auth::authenticate(&req)?;

    db::user::unfollow(&req.state().conn, &claims.username, &celeb_name)
        .await 
        .and_then(|profile| {
            //let profile: Profile = profile.into();
            Ok(json!(profile.wrap()).into())
        })
        .or_else(|err| err.into())
}

pub(crate) async fn create_article(mut req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authenticate(&req)?;

    let article_req: CreateArticleRequestWrapped = req.body_json().await?;

    db::article::create_article(&req.state().conn, &claims.username, &article_req.article)
        .await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn get_article(req: Request) -> tide::Result {
    let slug = req.param("slug")?;
    let filter = filters::ArticleFilterBySlug { slug };

    db::article::get_one(&req.state().conn, filter)
        .await 
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn get_articles(req: Request) -> tide::Result {
    let filter: filters::ArticleFilterByValues = req.query()?;
    let order_by = filters::OrderByFilter::Descending("updatedAt");
    let limit_offset: filters::LimitOffsetFilter = req.query()?;

    db::article::get_all(&req.state().conn, filter, order_by, limit_offset)
        .await 
        .and_then(|articles|
            Ok(json!(MultipleArticleResponse::from_articles(articles)).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn update_article(mut req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authenticate(&req)?;

    let update_article: UpdateArticleRequest = req.body_json().await?;
    let slug = req.param("slug")?;
    let filter = filters::UpdateArticleFilter { 
        slug,
        author: &claims.username
    };

    let res = match db::article::update_article(&req.state().conn, 
                                                update_article.article, 
                                                filter).await {
        Ok(article_response) => {
            Ok(json!(article_response.wrap()).into())
        },
        Err(err) => match err {
            // successful update returns the updated article, otherwise
            // NoArticleFound error is returned
            // if optimistic update fails, try to verify if this happened
            // because user is not authorized to do so
            crate::errors::BackendError::NoArticleFound => {
                let filter = filters::ArticleFilterBySlug { slug };
                if let Ok(article_response) = db::article::get_one(&req.state().conn, filter).await {

                    auth::Auth::authorize(&req, &article_response.article.author)
                        .and(
                            Err(tide::Error::from_str(
                                tide::StatusCode::InternalServerError, "could not update article despite user has been authorized, probably due to a bug"))
                        )

                } else {
                    crate::errors::BackendError::NoArticleFound.into()
                }
            },
            err @ _ =>  err.into(),
        }
    };
    res
}

pub(crate) async fn delete_article(mut req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authenticate(&req)?;

    let slug = req.param("slug")?;
    let filter = filters::UpdateArticleFilter { 
        slug,
        author: &claims.username
    };

    let query_res = db::article::delete_article(&req.state().conn, filter).await?;

    if 0 == query_res.rows_affected() {
        let filter = filters::ArticleFilterBySlug { slug };
        if let Ok(article_response) = db::article::get_one(&req.state().conn, filter).await {

            auth::Auth::authorize(&req, &article_response.article.author)
                .and(
                    Err(tide::Error::from_str(
                        tide::StatusCode::InternalServerError, "could not delete article despite user has been authorized, probably due to a bug"))
                )

        } else {
            crate::errors::BackendError::NoArticleFound.into()
        }
    } else {
        Ok(json!(()).into())
    }
}


pub(crate) async fn favorite_article(req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authenticate(&req)?;

    let slug = req.param("slug")?;
    let filter = filters::ArticleFilterBySlug { slug };

    db::article::favorite_article(&req.state().conn, filter, &claims.username)
        .await 
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn unfavorite_article(req: Request) -> tide::Result {
    let (claims, _) =  auth::Auth::authenticate(&req)?;

    let slug = req.param("slug")?;
    let filter = filters::ArticleFilterBySlug { slug };

    db::article::unfavorite_article(&req.state().conn, filter, &claims.username)
        .await 
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn feed_articles(req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authenticate(&req)?;

    let filter = filters::ArticleFilterFeed { follower: &claims.username };
    let order_by = filters::OrderByFilter::Descending("updatedAt");
    let limit_offset: filters::LimitOffsetFilter = req.query()?;

    db::article::get_all(&req.state().conn, filter, order_by, limit_offset)
        .await 
        .and_then(|articles|
            Ok(json!(MultipleArticleResponse::from_articles(articles)).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn add_comment(mut req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authenticate(&req)?;

    let wrapped: AddCommentRequestWrapped = req.body_json().await?;
    let slug = req.param("slug")?;
    let author = &claims.username;

    let filter = filters::ArticleFilterBySlug { slug };

    db::article::add_comment(&req.state().conn, filter, author, &wrapped.comment)
        .await 
        .and_then(|comment|
          Ok(json!(comment.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn get_comments(req: Request) -> tide::Result {
    let mut filter = filters::CommentFilterByValues { 
        id: None,
        article_slug: Some(req.param("slug")?),
        author: None
    };

    let tmp = auth::Auth::authenticate(&req).ok().and_then(|(claims, _)| Some(claims.username));
    // author is an Option, can be None
    filter.author = tmp.as_deref();

    let order_by = filters::OrderByFilter::Descending("id");
    let limit_offset: filters::LimitOffsetFilter = filters::LimitOffsetFilter::default();

    db::article::get_comments(&req.state().conn, filter, order_by, limit_offset)
        .await 
        .and_then(|comments|
          Ok(json!(MultipleCommentResponse { comments }).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn delete_comment(req: Request) -> tide::Result {
    let (claims, _) = auth::Auth::authenticate(&req)?;
    
    let id = req.param("id")?.parse::<i32>()?;

    let filter = filters::CommentFilterByValues::default().id(id).author(&claims.username);

    let query_res = db::article::delete_comments(&req.state().conn, filter).await?;
    // if no comment has been deleted, check if user is authorized to do so    
    if 0 == query_res.rows_affected() {
        let filter = filters::CommentFilterByValues::default().id(id); 

        let comments = db::article::get_comments(&req.state().conn, 
                                                filter, 
                                                filters::OrderByFilter::default(), 
                                                filters::LimitOffsetFilter::default().limit(1))
            .await; 

        match comments {
            Ok(comments) =>
                if let Some(comment) = comments.iter().next() {

                    auth::Auth::authorize(&req, &comment.comment.author)
                        .and(
                            Err(tide::Error::from_str(
                                tide::StatusCode::InternalServerError, "could not delete comment despite user has been authorized, probably due to a bug"))
                        )

                } else {
                    crate::errors::BackendError::NoCommentFound(id).into()
                },
            Err(err) => err.into()
        }
    } else {
        Ok(json!(()).into())
    }

}

pub(crate) async fn get_tags(req: Request) -> tide::Result {

    db::article::get_tags(&req.state().conn)
        .await 
        .and_then(|tags|
            Ok(json!(tags).into())
        )
        .or_else(|err| err.into())
}
