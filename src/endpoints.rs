use tide::prelude::*;

//use validator::{Validate};
use crate::app::AppState;
use crate::{models::{user::*, article::*}, filters};

pub(crate) type Request = tide::Request<AppState>;

pub(crate) async fn register(mut req: Request) -> tide::Result {
    let wrapped: UserRegWrapped = req.body_json().await?;

    req.state().server.register_user(wrapped.user).await
        .and_then(|user| 
            Ok(json!(user.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn login(mut req: Request) -> tide::Result {
    println!("in login");
    let login_req: LoginRequestWrapped = req.body_json().await?;
    let filter = filters::UserFilter::default().email(&login_req.user.email);

    req.state().server.login_user(filter).await
        .and_then(|user| 
            Ok(json!(user.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn current_user(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;

    req.state().server.user_by_token(token).await
        .and_then(|user| 
            Ok(json!(user.wrap()).into())
        )
        .or_else(|err| err.into())
/*
    let secret = req.state().secret;
    let claims =   auth::Auth::authenticate(token, secret)?;
    
    let filter = filters::UserFilter::default().username(&claims.username);
    
    db::user::get_user(&req.state().server.conn, filter)
        .await
        .and_then(|user| {
                let mut user: User = user.into();
                user.token = Some(token.to_string());
                Ok(json!(user.wrap()).into())
        })
        .or_else(|err| err.into())*/
}
              
pub(crate) async fn profile(req: Request) -> tide::Result {
    let username = req.param("username")?;
    req.state().server.profile(username).await
        .and_then(|profile| 
            Ok(json!(profile.wrap()).into())
        )
        .or_else(|err| err.into())

//    let filter = filters::UserFilter::default().username(&username);
/*
    let res = match db::user::get_profile(&req.state().server.conn, &username).await {
        Some(profile) => {
            Ok(json!(profile.wrap()).into())
        },
        None => 
            crate::errors::BackendError::NoUserFound(username.to_string()).into()
    };
    res   */       
}

pub(crate) async fn update_user(mut req: Request) -> tide::Result {
    let update_user: UserUpdateWrapped = req.body_json().await?;
    let token = crate::utils::token_from_request(&req)?;

    req.state().server.update_user(token, update_user.user).await
        .and_then(|user| 
            Ok(json!(user.wrap()).into())
        )
        .or_else(|err| err.into())

/*
    let secret = req.state().secret;
    let claims =   auth::Auth::authenticate(token, secret)?;
    let token = token.to_string();

    let update_user: UserUpdateWrapped = req.body_json().await?;
    let filter = filters::UpdateUserFilter::default().username(&claims.username);

    db::user::update_user(&req.state().server.conn, &update_user.user, filter)
        .await
        .and_then(|user| {
            let mut user: User = user.into();
            user.token = Some(token);
            Ok(json!(user.wrap()).into())
        })
        .or_else(|err| err.into())*/
}

pub(crate) async fn follow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;
    let token = crate::utils::token_from_request(&req)?;

    req.state().server.follow(token, celeb_name).await
        .and_then(|profile| 
            Ok(json!(profile.wrap()).into())
        )
        .or_else(|err| err.into())
/*

    let secret = req.state().secret;
    let claims =   auth::Auth::authenticate(token, secret)?;

    db::user::follow(&req.state().server.conn, &claims.username, &celeb_name)
        .await
        .and_then(|profile| {
            let profile: Profile = profile.into();
            Ok(json!(profile.wrap()).into())
        })
        .or_else(|err| err.into())*/
}

pub(crate) async fn unfollow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;
    let token = crate::utils::token_from_request(&req)?;

    req.state().server.unfollow(token, celeb_name).await
        .and_then(|profile| 
            Ok(json!(profile.wrap()).into())
        )
        .or_else(|err| err.into())
/*
    let celeb_name = req.param("username")?;

    let token = crate::utils::token_from_request(&req)?;
    let secret = req.state().secret;
    let claims =   auth::Auth::authenticate(token, secret)?;

    db::user::unfollow(&req.state().server.conn, &claims.username, &celeb_name)
        .await 
        .and_then(|profile| {
            //let profile: Profile = profile.into();
            Ok(json!(profile.wrap()).into())
        })
        .or_else(|err| err.into())*/
}

pub(crate) async fn create_article(mut req: Request) -> tide::Result {
    let article_req_wrapped: CreateArticleRequestWrapped = req.body_json().await?;
    let token = crate::utils::token_from_request(&req)?;

    req.state().server.create_article(token, &article_req_wrapped.article).await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
/*

    let secret = req.state().secret;
    let claims =   auth::Auth::authenticate(token, secret)?;

    let article_req: CreateArticleRequestWrapped = req.body_json().await?;

    db::article::create_article(&req.state().server.conn, &claims.username, &article_req.article)
        .await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())*/
}

pub(crate) async fn get_article(req: Request) -> tide::Result {
    let slug = req.param("slug")?;
    let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());

    req.state().server.get_article(filter).await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
/*
    db::article::get_one(&req.state().server.conn, filter)
        .await 
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())*/
}

pub(crate) async fn get_articles(req: Request) -> tide::Result {
    let filter: filters::ArticleFilterByValues = req.query()?;
    let order_by = filters::OrderByFilter::Descending("updatedAt");
    let limit_offset: filters::LimitOffsetFilter = req.query()?;

    req.state().server.get_articles(filter, order_by, limit_offset).await
        .and_then(|multiple_articles_response| 
            Ok(json!(multiple_articles_response).into())
        )
        .or_else(|err| err.into())
/*
    db::article::get_all(&req.state().server.conn, filter, order_by, limit_offset)
        .await 
        .and_then(|articles|
            Ok(json!(MultipleArticleResponse::from_articles(articles)).into())
        )
        .or_else(|err| err.into())*/
}

pub(crate) async fn update_article(mut req: Request) -> tide::Result {
    let update_article: UpdateArticleRequest = req.body_json().await?;
    let token = crate::utils::token_from_request(&req)?;

    let slug = req.param("slug")?;
 /*   let filter = filters::UpdateArticleFilter { 
        slug,
        author: &claims.username
    };*/

    req.state().server.update_article(token, update_article.article, slug).await
        .and_then(|articles_response| 
            Ok(json!(articles_response.wrap()).into())
        )
        .or_else(|err| err.into())
/*
    let secret = req.state().secret;
    let claims =   auth::Auth::authenticate(token, secret)?;


    let res = match db::article::update_article(&req.state().server.conn, 
                                                token,
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
                let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());
                if let Ok(article_response) = db::article::get_one(&req.state().server.conn, filter).await {

                    let token = crate::utils::token_from_request(&req)?;
                    let secret = req.state().secret;
                    auth::Auth::authorize(token, secret, &article_response.article.author)
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
    res*/
}

pub(crate) async fn delete_article(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;
    let slug = req.param("slug")?;

    req.state().server.delete_article(token, slug).await
        .and_then(|()| 
            Ok(json!(()).into())
        )
        .or_else(|err| err.into())
/*
    let filter = filters::UpdateArticleFilter { 
        slug,
        author: &claims.username
    };

    let query_res = db::article::delete_article(&req.state().server.conn, filter).await?;

    if 0 == query_res.rows_affected() {
        let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());
        if let Ok(article_response) = db::article::get_one(&req.state().server.conn, filter).await {

            auth::Auth::authorize(token, secret, &article_response.article.author)
                .and(
                    Err(tide::Error::from_str(
                        tide::StatusCode::InternalServerError, "could not delete article despite user has been authorized, probably due to a bug"))
                )

        } else {
            crate::errors::BackendError::NoArticleFound.into()
        }
    } else {
        Ok(json!(()).into())
    }*/
}


pub(crate) async fn favorite_article(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;

    let slug = req.param("slug")?;
    let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());

    req.state().server.favorite_article(token, filter).await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
/*
    db::article::favorite_article(&req.state().server.conn, filter, &claims.username)
        .await 
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())*/
}

pub(crate) async fn unfavorite_article(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;

    let slug = req.param("slug")?;
    let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());

    req.state().server.unfavorite_article(token, filter).await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
/*
    let token = crate::utils::token_from_request(&req)?;
    let secret = req.state().secret;
    let claims =   auth::Auth::authenticate(token, secret)?;

    let slug = req.param("slug")?;
    let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());

    db::article::unfavorite_article(&req.state().server.conn, filter, &claims.username)
        .await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())*/
}

pub(crate) async fn feed_articles(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;
    let limit_offset: filters::LimitOffsetFilter = req.query()?;

    req.state().server.feed_articles(token, limit_offset).await
        .and_then(|articles| 
            Ok(json!(articles).into())
        )
        .or_else(|err| err.into())

//    let secret = req.state().secret;
//    let claims =   auth::Auth::authenticate(token, secret)?;
/*
    let filter = filters::ArticleFilterFeed { follower: &claims.username };
    let order_by = filters::OrderByFilter::Descending("updatedAt");

    db::article::get_all(&req.state().server.conn, filter, order_by, limit_offset)
        .await 
        .and_then(|articles|
            Ok(json!(MultipleArticleResponse::from_articles(articles)).into())
        )
        .or_else(|err| err.into())*/
}

pub(crate) async fn add_comment(mut req: Request) -> tide::Result {
    let wrapped: AddCommentRequestWrapped = req.body_json().await?;
    let token = crate::utils::token_from_request(&req)?;
    let slug = req.param("slug")?;

    let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());

    req.state().server.add_comment(token, filter, wrapped.comment).await
        .and_then(|comment| 
            Ok(json!(comment.wrap()).into())
        )
        .or_else(|err| err.into())

/*    db::article::add_comment(&req.state().server.conn, filter, author, &wrapped.comment)
        .await 
        .and_then(|comment|
          Ok(json!(comment.wrap()).into())
        )
        .or_else(|err| err.into())*/
}

pub(crate) async fn get_comments(req: Request) -> tide::Result {
    let mut filter = filters::CommentFilterByValues { 
        id: None,
        article_slug: Some(req.param("slug")?),
        author: None
    };
    let order_by = filters::OrderByFilter::Descending("id");
    let limit_offset: filters::LimitOffsetFilter = filters::LimitOffsetFilter::default();

    req.state().server.get_comments(filter, order_by, limit_offset).await
        .and_then(|comments| 
            Ok(json!(comments).into())
        )
        .or_else(|err| err.into())

/*
    let token_opt = crate::utils::token_from_request(&req).ok();
    let secret = req.state().secret;

    let tmp =  auth::Auth::authenticate(token, secret).ok().and_then(|claims| Some(claims.username));
    // author is an Option, can be None
    filter.author = tmp.as_deref();

    let order_by = filters::OrderByFilter::Descending("id");
    let limit_offset: filters::LimitOffsetFilter = filters::LimitOffsetFilter::default();

    db::article::get_comments(&req.state().server.conn, filter, order_by, limit_offset)
        .await 
        .and_then(|comments|
          Ok(json!(MultipleCommentResponse { comments }).into())
        )
        .or_else(|err| err.into())
        */
}

pub(crate) async fn delete_comment(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;
    let id = req.param("id")?.parse::<i32>()?;

    let filter = filters::CommentFilterByValues::default().id(id);
    req.state().server.delete_comment(token, filter).await
        .and_then(|()| 
            Ok(json!(()).into())
        )
        .or_else(|err| err.into())
/*
    let secret = req.state().secret;
    let claims =   auth::Auth::authenticate(token, secret)?;
    

    let filter = filters::CommentFilterByValues::default().id(id).author(&claims.username);

    let query_res = db::article::delete_comments(&req.state().server.conn, filter).await?;
    // if no comment has been deleted, check if user is authorized to do so    
    if 0 == query_res.rows_affected() {
        let filter = filters::CommentFilterByValues::default().id(id); 

        let comments = db::article::get_comments(&req.state().server.conn, 
                                                filter, 
                                                filters::OrderByFilter::default(), 
                                                filters::LimitOffsetFilter::default().limit(1))
            .await; 

        match comments {
            Ok(comments) =>
                if let Some(comment) = comments.iter().next() {

                    auth::Auth::authorize(token, secret, &comment.comment.author)
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
*/
}

pub(crate) async fn get_tags(req: Request) -> tide::Result {

    req.state().server.get_tags()
        .await 
        .and_then(|tags|
            Ok(json!(tags).into())
        )
        .or_else(|err| err.into())
}
