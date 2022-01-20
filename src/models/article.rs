use tide::prelude::*;
use slugify::slugify;
use crate::utils::*;
use crate::requests::article::*;

#[derive(sqlx::FromRow)]
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub struct Article { 
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    #[serde(serialize_with = "transform_string_to_vec")]    
    pub tag_list: Option<String>,
    #[serde(serialize_with = "transform_datetime_option")]    
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(serialize_with = "transform_datetime_option")]    
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub favorited: bool,
    pub favorites_count: u32,
    #[serde(skip_serializing)]
    pub author: String,
}

impl From<CreateArticleRequestAuthenticated> for Article {
    fn from(create_article: CreateArticleRequestAuthenticated) -> Self {
        Self { 
            slug: create_article.article_request.slug.clone(), 
            title: create_article.article_request.title.clone(),
            description: create_article.article_request.description.clone(),
            body: create_article.article_request.body.clone(),
            tag_list: create_article.article_request.tag_list.as_ref()
                .and_then(|tags| 
                    Some( tags
                            .iter()
                            .fold("".to_string(), |s, tag| format!("{}{},", s, tag) ) )
                ),
            created_at: None,
            updated_at: None,
            favorited: false,
            favorites_count: 0,
            author: create_article.author.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub struct UpdateArticle { 
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
}

impl UpdateArticle {
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
    pub fn get_slug(&self) -> Option<String> {
        self.title.as_ref().and_then(|title| Some(slugify!(title)))
    }
}
/*
impl UpdateArticle {
    pub fn updated_slug(&self) -> Option<&str> {
        self.slug_from_title.as_deref()
    }
}
*/
impl std::fmt::Display for UpdateArticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.title.as_ref().map(|val| 
//            write!( f, " {}='{}' , {}='{}'", "title", val, "slug", self.slug_from_title.as_ref().unwrap()) ).unwrap_or(Ok(()))?;
            write!( f, " {}='{}', {}='{}', ", "title", val, "slug", slugify!(val)) ).unwrap_or(Ok(()))?;
        self.description.as_ref().map(|val| write!( f, " {}='{}', ", "description", val) ).unwrap_or(Ok(()))?;
        self.body.as_ref().map(|val| write!( f, " {}='{}', ", "body", val) ).unwrap_or(Ok(()))?;
        write!( f, " id=id ")
    }
}

impl Default for UpdateArticle {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            body: None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ArticleResponse {
    pub author: Option<super::user::Profile>,
    #[serde(flatten)]
    pub article: Article,
}

impl ArticleResponse {
    pub(crate) fn wrap(self) -> ArticleResponseWrapped {
        ArticleResponseWrapped { article: self }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ArticleResponseWrapped {
    pub article: ArticleResponse,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MultipleArticleResponse {
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


#[derive(sqlx::FromRow)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub struct Comment { 
    pub id: i32,
    #[serde(serialize_with = "transform_datetime")]    
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(serialize_with = "transform_datetime")]    
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub body: String,
    #[serde(skip_serializing)]
    pub author: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CommentResponse {
    pub author: Option<crate::models::user::Profile>,
    #[serde(flatten)]
    pub comment: Comment,
}

impl CommentResponse {
    pub(crate) fn wrap(self) -> CommentResponseWrapped {
        CommentResponseWrapped {comment: self}
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct CommentResponseWrapped {
    pub comment: CommentResponse,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MultipleCommentResponse {
    pub comments: Vec<CommentResponse>,
}

impl MultipleCommentResponse {
    pub fn from_comments(comments: Vec<CommentResponse>) -> Self {
        Self { 
            comments,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagList {
    pub tags: Vec<String>,
}

impl std::fmt::Display for TagList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!( f, "[")?;
        for tag in &self.tags {
            write!( f, "{}, ", tag)?;
        }
        write!( f, "]")
    }
}



