use tide::prelude::*;

use crate::app::AppState;
use crate::{requests::{user::*, article::*}, filters};

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
    let login_req: LoginRequestWrapped = req.body_json().await?;

    req.state().server.login_user(login_req.user).await
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
}
              
pub(crate) async fn profile(req: Request) -> tide::Result {
    let username = req.param("username")?;
    req.state().server.profile(username).await
        .and_then(|profile| 
            Ok(json!(profile.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn update_user(mut req: Request) -> tide::Result {
    let update_user: UserUpdateWrapped = req.body_json().await?;
    let token = crate::utils::token_from_request(&req)?;

    req.state().server.update_user(token, update_user.user).await
        .and_then(|user| 
            Ok(json!(user.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn follow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;
    let token = crate::utils::token_from_request(&req)?;

    req.state().server.follow(token, celeb_name).await
        .and_then(|profile| 
            Ok(json!(profile.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn unfollow(req: Request) -> tide::Result {
    let celeb_name = req.param("username")?;
    let token = crate::utils::token_from_request(&req)?;

    req.state().server.unfollow(token, celeb_name).await
        .and_then(|profile| 
            Ok(json!(profile.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn create_article(mut req: Request) -> tide::Result {
    let article_req_wrapped: CreateArticleRequestWrapped = req.body_json().await?;
    let token = crate::utils::token_from_request(&req)?;

    req.state().server.create_article(token, article_req_wrapped.article).await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn get_article(req: Request) -> tide::Result {
    let slug = req.param("slug")?;

    req.state().server.get_article_by_slug(slug).await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
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
}

pub(crate) async fn update_article(mut req: Request) -> tide::Result {
    let update_article_req_body: UpdateArticleRequestBody = req.body_json().await?;
    let token = crate::utils::token_from_request(&req)?;

    let slug = req.param("slug")?;
    let update_article_req = UpdateArticleRequest::from_req_parts(update_article_req_body, slug);

    req.state().server.update_article(token, update_article_req).await
        .and_then(|articles_response| 
            Ok(json!(articles_response.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn delete_article(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;
    let slug = req.param("slug")?;

    req.state().server.delete_article(token, slug).await
        .and_then(|()| 
            Ok(json!(()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn favorite_article(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;

    let slug = req.param("slug")?;

    req.state().server.favorite_article(token, slug).await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn unfavorite_article(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;

    let slug = req.param("slug")?;

    req.state().server.unfavorite_article(token, slug).await
        .and_then(|article_response| 
            Ok(json!(article_response.wrap()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn feed_articles(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;
    let limit_offset: filters::LimitOffsetFilter = req.query()?;

    req.state().server.feed_articles(token, limit_offset).await
        .and_then(|articles| 
            Ok(json!(articles).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn get_comments(req: Request) -> tide::Result {
    let slug = req.param("slug")?;

    req.state().server.get_comments(slug).await
        .and_then(|comments| 
            Ok(json!(comments).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn add_comment(mut req: Request) -> tide::Result {
    let wrapped: AddCommentRequestBodyWrapped = req.body_json().await?;
    let article_slug = req.param("slug")?;
    let token = crate::utils::token_from_request(&req)?;

    let add_comment_req = AddCommentRequest::from_req_parts(wrapped, article_slug);

    req.state().server.add_comment(token, add_comment_req).await
        .and_then(|comment| 
            Ok(json!(comment.wrap()).into())
        )
        .or_else(|err| err.into())
}


pub(crate) async fn delete_comment(req: Request) -> tide::Result {
    let token = crate::utils::token_from_request(&req)?;
    let id = req.param("id")?.parse::<i32>()?;
    let article_slug = req.param("slug")?;

    let delete_comment_req = crate::requests::article::DeleteCommentRequest { 
        id,
        article_slug
    };
    req.state().server.delete_comment(token, delete_comment_req).await
        .and_then(|()| 
            Ok(json!(()).into())
        )
        .or_else(|err| err.into())
}

pub(crate) async fn get_tags(req: Request) -> tide::Result {
    req.state().server.get_tags()
        .await 
        .and_then(|tags|
            Ok(json!(tags).into())
        )
        .or_else(|err| err.into())
}
