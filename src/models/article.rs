#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub(crate) struct CreateArticleRequest { 
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

#[derive(sqlx::FromRow)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub(crate) struct Comment { 
    pub id: i32,
    #[serde(serialize_with = "transform_datetime")]    
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(serialize_with = "transform_datetime")]    
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub body: String,
    #[serde(skip_serializing)]
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CommentResponse {
    pub author: Option<crate::models::user::Profile>,
    #[serde(flatten)]
    pub comment: Comment,
}

#[derive(Debug, Serialize)]
pub(crate) struct CommentResponseWrapped {
    pub comment: CommentResponse,
}
/*
impl CommentResponseWrapped {
    fn from_comment(comment: CommentResponse) -> Self {
        Self { comment }
    }
}
*/

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MultipleCommentResponse {
    pub comments: Vec<CommentResponse>,
}

impl MultipleCommentResponse {
    pub fn from_comments(comments: Vec<CommentResponse>) -> Self {
        Self { 
            comments,
        }
    }
}


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub(crate) struct AddCommentRequest { 
    pub body: String,
}
use tide::prelude::*;
use sqlx::prelude::*;
use crate::utils::*;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct AddCommentRequestWrapped { 
    pub comment: AddCommentRequest,
}

#[derive(sqlx::FromRow)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub(crate) struct Article { 
//    #[serde(skip_serializing)]
    pub slug: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    #[serde(serialize_with = "transform_string_to_vec")]    
    pub tag_list: Option<String>,
    #[serde(serialize_with = "transform_datetime")]    
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(serialize_with = "transform_datetime")]    
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub favorited: bool,
    pub favorites_count: u32,
    #[serde(skip_serializing)]
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ArticleResponse {
    pub author: Option<super::user::Profile>,
    #[serde(flatten)]
    pub article: Article,
}


#[derive(Debug, Serialize)]
pub(crate) struct ArticleResponseWrapped {
    pub article: ArticleResponse,
}
/*
impl ArticleResponseWrapped {
    fn from_article(article: ArticleResponse) -> Self {
        Self { article }
    }
}
*/
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MultipleArticleResponse {
    pub articles: Vec<ArticleResponse>,
    articles_count: usize,
}

impl MultipleArticleResponse {
    pub fn from_articles(articles: Vec<ArticleResponse>) -> Self {
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub(crate) struct UpdateArticleRequest { 
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    #[serde(skip_deserializing)]
    pub slug_from_title: Option<String>,
}

