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

pub(crate) struct CreateArticleRequestAuthenticated<'a> {
    pub article_request: &'a CreateArticleRequest,
    pub author: &'a str,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub(crate) struct UpdateArticleRequest {
//    #[serde(deserialize_with = "slugify_article_on_update")]    
    pub article: UpdateArticle,
}



pub struct DeleteCommentRequest<'a> {
    pub id: i32,
    pub article_slug: &'a str,
}
/*
impl<'a> DeleteCommentRequest<'a> {
    pub(crate) fn authenticate(self, author: &'a str) -> DeleteCommentRequestAuthenticated<'a> {
        DeleteCommentRequestAuthenticated {
            article_request: self,
            author
        }
    }
}
*/

impl<'a> IntoAuthenticatedRequest<DeleteCommentRequestAuthenticated<'a>> for DeleteCommentRequest<'a> {
}

pub(crate) struct DeleteCommentRequestAuthenticated<'a> {
    pub article_request: DeleteCommentRequest<'a>,
    pub author: String,
}

impl<'a> AuthenticatedRequest for DeleteCommentRequestAuthenticated<'a> {
    type FromRequest = DeleteCommentRequest<'a>;
    fn from_request_with_claims(req: DeleteCommentRequest<'a>, 
                                claims: crate::auth::Claims) -> Self {
        Self {
            article_request: req,
            author: claims.username,
        }
    }
}

