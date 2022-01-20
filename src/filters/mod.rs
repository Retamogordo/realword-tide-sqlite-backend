use tide::prelude::*;
use crate::requests::{article::*};

pub trait Filter: std::fmt::Display + Default {}

pub(crate) struct UserFilter<'a> {
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password: Option<&'a str>,
}

impl<'a> UserFilter<'a> {
    pub fn username(mut self, username: &'a str) -> Self {
        self.username = Some(username);
        self
    } 
    pub fn email(mut self, email: &'a str) -> Self {
        self.email = Some(email);
        self
    } 
    pub fn password(mut self, password: &'a str) -> Self {
        self.password = Some(password);
        self
    } 
}
impl Filter for UserFilter<'_> {}

impl Default for UserFilter <'_>{
    fn default() -> Self {
        Self { 
            username: None,
            email: None,
            password: None,
        }
    }
}

impl std::fmt::Display for UserFilter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.username.as_ref().map(|val| write!( f, " {}='{}' AND", "users.username", val) ).unwrap_or(Ok(()))?;
        self.email.as_ref().map(|val| write!( f, " {}='{}' AND", "users.email", val) ).unwrap_or(Ok(()))?;
        self.password.as_ref().map(|val| write!( f, " {}='{}' AND", "users.password", val) ).unwrap_or(Ok(()))?;
        write!( f, " 1=1")
    }
}

pub struct UpdateUserFilter<'a> {
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
}

impl<'a> UpdateUserFilter<'a> {
    pub fn username(mut self, username: &'a str) -> Self {
        self.username = Some(username);
        self
    }
    pub fn email(mut self, email: &'a str) -> Self {
        self.email = Some(email);
        self
    }
}

impl Filter for UpdateUserFilter<'_> {}

impl std::fmt::Display for UpdateUserFilter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.username.as_ref().map(|val| write!( f, " {}='{}' AND", "users.username", val) ).unwrap_or(Ok(()))?;
        self.email.as_ref().map(|val| write!( f, " {}='{}' AND", "users.email", val) ).unwrap_or(Ok(()))?;
        write!( f, " 1=1")
    }
}
impl Default for UpdateUserFilter<'_> {
    fn default() -> Self {
        Self { 
            username: None,
            email: None,
        }
    }
}

/*
#[derive(Deserialize)]
pub(crate) struct ArticleFilterBySlug<'a> {
    pub slug: &'a str,
}

impl Filter for ArticleFilterBySlug<'_> {}
impl std::fmt::Display for ArticleFilterBySlug<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!( f, " {}='{}'", "slug", self.slug)
    }
}
*/
#[derive(Deserialize)]
#[serde(default)]
pub struct ArticleFilterByValues {
    pub author: Option<String>,
    pub tag: Option<String>,
    pub favorited: Option<String>,
    pub slug: Option<String>,
}

impl ArticleFilterByValues {
    pub fn author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }
    pub fn slug(mut self, slug: String) -> Self {
        self.slug = Some(slug);
        self
    }
    pub fn tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);
        self
    }
    pub fn favorited(mut self, favorited: String) -> Self {
        self.favorited = Some(favorited);
        self
    }
}

impl Filter for ArticleFilterByValues {}

impl Default for ArticleFilterByValues {
    fn default() -> Self {
        Self { 
            author: None,
            tag: None,
            favorited: None,
            slug: None,
        }
    }
}

impl std::fmt::Display for ArticleFilterByValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.author.as_ref().map(|val| write!( f, " {}='{}' AND", "author", val) ).unwrap_or(Ok(()))?;
        self.slug.as_ref().map(|val| write!( f, " {}='{}' AND", "slug", val) ).unwrap_or(Ok(()))?;
        self.tag.as_ref().map(|val| write!( f, " {} LIKE '%{}%' AND", "tagList", val) ).unwrap_or(Ok(()))?;
        self.favorited.as_ref().map(|val| 
            write!( f, " {}='{}' AND", "favorite_articles.username", val) 
        ).unwrap_or(Ok(()))?;
        write!( f, " 1=1")
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct ArticleFilterFeed<'a> {
    pub follower: &'a str,
}

impl Filter for ArticleFilterFeed<'_> {}

impl std::fmt::Display for ArticleFilterFeed<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!( f, 
            " {} IN (SELECT celeb_name FROM followers WHERE follower_name='{}')", 
            "author", self.follower)
    }
}


#[derive(Default)]
pub(crate) struct UpdateArticleFilter<'a> {
    pub slug: &'a str,
    pub author: &'a str,
}
impl Filter for UpdateArticleFilter<'_> {}

impl std::fmt::Display for UpdateArticleFilter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!( f, " slug='{}' AND author='{}'", self.slug, self.author)
    }
}

impl<'a> From<&'a DeleteArticleRequestAuthenticated<'_>> for UpdateArticleFilter<'a> {
    fn from(req: &'a DeleteArticleRequestAuthenticated) -> Self {
        Self {
            slug: req.article_request.slug,
            author: &req.author,
        }
    }
}

impl<'a> From<&'a UpdateArticleRequestAuthenticated<'_>> for UpdateArticleFilter<'a> {
    fn from(req: &'a UpdateArticleRequestAuthenticated) -> Self {
        Self {
            slug: req.article_request.slug,
            author: &req.author,
        }
    }
}

pub enum OrderByFilter<'a> {
    #[allow(dead_code)]
    Ascending(&'a str),
    Descending(&'a str),
    None,
}

impl std::fmt::Display for OrderByFilter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {         
        match self {
            Self::Ascending(row_name) => write!( f, "ORDER BY {} ASC ", row_name),
            Self::Descending(row_name) => write!( f, "ORDER BY {} DESC ", row_name),
            Self::None => Ok(()),
        }
    }
}

impl Default for OrderByFilter<'_> {
    fn default() -> Self {
        Self::None
    }
}

pub struct CommentFilterByValues<'a> {
    pub id: Option<i32>,
    pub author: Option<&'a str>,
    pub article_slug: Option<&'a str>,
}

impl<'a> CommentFilterByValues<'a> {
    pub fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }
    pub fn author(mut self, author: &'a str) -> Self {
        self.author = Some(author);
        self
    }
    pub fn article_slug(mut self, article_slug: &'a str) -> Self {
        self.article_slug = Some(article_slug);
        self
    }
}

impl Default for CommentFilterByValues<'_> {
    fn default() -> Self {
        Self { 
            id: None,
            author: None,
            article_slug: None,
        }
    }
}

impl<'a> From<&'a AddCommentRequestAuthenticated<'_>> for CommentFilterByValues<'a> {
    fn from(req: &'a AddCommentRequestAuthenticated) -> Self {
        Self {
            id: None,
            article_slug: Some(req.article_request.article_slug),
            author: Some(&req.author),
        }
    }
}
impl<'a> From<&'a DeleteCommentRequestAuthenticated<'_>> for CommentFilterByValues<'a> {
    fn from(req: &'a DeleteCommentRequestAuthenticated) -> Self {
        Self {
            id: Some(req.article_request.id),
            article_slug: Some(req.article_request.article_slug),
            author: Some(&req.author),
        }
    }
}

impl std::fmt::Display for CommentFilterByValues<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.as_ref().map(|val| write!( f, " {}='{}' AND", "id", val) ).unwrap_or(Ok(()))?;
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
pub struct LimitOffsetFilter {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl LimitOffsetFilter {
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }
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

