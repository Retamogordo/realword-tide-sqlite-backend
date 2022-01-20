use tide::prelude::*;
use crate::models::article::*;
use crate::requests::{AuthenticatedRequest, IntoAuthenticatedRequest};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub struct CreateArticleRequest { 
    #[serde(skip_deserializing)]
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub tag_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct CreateArticleRequestWrapped { 
    #[serde(deserialize_with = "slugify_article_on_create")]    
    pub article: CreateArticleRequest,
}

fn slugify_article_on_create<'de, D>(deserializer: D) -> std::result::Result<CreateArticleRequest, D::Error>
where
    D: serde::Deserializer<'de>, {
    use slugify::slugify;
    let mut req: CreateArticleRequest = serde::Deserialize::deserialize(deserializer)?;
    req.slug = slugify!(&req.title);
    Ok(req)
}

pub(crate) struct CreateArticleRequestAuthenticated {
    pub article_request: CreateArticleRequest,
    pub author: String,
}

impl IntoAuthenticatedRequest<CreateArticleRequestAuthenticated> for CreateArticleRequest {
}

impl AuthenticatedRequest for CreateArticleRequestAuthenticated {
    type FromRequest = CreateArticleRequest;
    fn from_request_with_claims(req: Self::FromRequest, 
                                claims: crate::auth::Claims) -> Self {
        Self {
            article_request: req,
            author: claims.username,
        }
    }
}


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub(crate) struct UpdateArticleRequestBody {
    pub article: UpdateArticle,
}

pub struct UpdateArticleRequest<'a> {
    pub article: UpdateArticle,
    pub slug: &'a str,
}

impl <'a> UpdateArticleRequest<'a> {
    pub(crate) fn from_req_parts(body: UpdateArticleRequestBody, slug: &'a str) -> Self {
        Self {
            article: body.article,
            slug,
        }
    }
}

pub(crate) struct UpdateArticleRequestAuthenticated<'a> {
    pub article_request: UpdateArticleRequest<'a>,
    pub author: String,
}

impl<'a> IntoAuthenticatedRequest<UpdateArticleRequestAuthenticated<'a>> for UpdateArticleRequest<'a> {
}

impl<'a> AuthenticatedRequest for UpdateArticleRequestAuthenticated<'a> {
    type FromRequest = UpdateArticleRequest<'a>;
    fn from_request_with_claims(req: Self::FromRequest, claims: crate::auth::Claims) -> Self {
        Self {
            article_request: req,
            author: claims.username,
        }
    }
}


pub struct DeleteArticleRequest<'a> { 
    pub slug: &'a str,
}

impl<'a> IntoAuthenticatedRequest<DeleteArticleRequestAuthenticated<'a>> for DeleteArticleRequest<'a> {
}

pub(crate) struct DeleteArticleRequestAuthenticated<'a> {
    pub article_request: DeleteArticleRequest<'a>,
    pub author: String,
}

impl<'a> AuthenticatedRequest for DeleteArticleRequestAuthenticated<'a> {
    type FromRequest = DeleteArticleRequest<'a>;
    fn from_request_with_claims(req: Self::FromRequest, claims: crate::auth::Claims) -> Self {
        Self {
            article_request: req,
            author: claims.username,
        }
    }
}


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
struct AddCommentRequestBody { 
    pub body: String,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct AddCommentRequestBodyWrapped { 
    comment: AddCommentRequestBody,
}

pub struct AddCommentRequest<'a> {
    pub body: String,
    pub article_slug: &'a str,
}

impl<'a> AddCommentRequest<'a> {
    pub(crate) fn from_req_parts(
        body_wrapped: AddCommentRequestBodyWrapped, 
        article_slug: &'a str) -> Self {
            Self { 
                body: body_wrapped.comment.body,
                article_slug
            }
    }
}

impl<'a> IntoAuthenticatedRequest<AddCommentRequestAuthenticated<'a>> for AddCommentRequest<'a> {
}

pub(crate) struct AddCommentRequestAuthenticated<'a> {
    pub article_request: AddCommentRequest<'a>,
    pub author: String,
}

impl<'a> AuthenticatedRequest for AddCommentRequestAuthenticated<'a> {
    type FromRequest = AddCommentRequest<'a>;
    fn from_request_with_claims(req: Self::FromRequest, claims: crate::auth::Claims) -> Self {
        Self {
            article_request: req,
            author: claims.username,
        }
    }
}



pub struct DeleteCommentRequest<'a> {
    pub id: i32,
    pub article_slug: &'a str,
}

impl<'a> IntoAuthenticatedRequest<DeleteCommentRequestAuthenticated<'a>> for DeleteCommentRequest<'a> {
}

pub(crate) struct DeleteCommentRequestAuthenticated<'a> {
    pub article_request: DeleteCommentRequest<'a>,
    pub author: String,
}

impl<'a> AuthenticatedRequest for DeleteCommentRequestAuthenticated<'a> {
    type FromRequest = DeleteCommentRequest<'a>;
    fn from_request_with_claims(req: Self::FromRequest, claims: crate::auth::Claims) -> Self {
        Self {
            article_request: req,
            author: claims.username,
        }
    }
}

