//use tide::{Response, Next, Result, StatusCode};
use tide::prelude::*;

//use sqlx::prelude::*;

//use validator::{Validate};
//use async_std::fs::File;
//use async_std::io::ReadExt;

pub(crate) trait ArticleFilter: std::fmt::Display {}

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
//#[serde(default)]
pub(crate) struct UpdateArticleFilter<'a> {
    #[serde(skip_deserializing)]
    pub slug: &'a str,
    #[serde(skip_deserializing)]
    pub author: &'a str,
    #[serde(deserialize_with = "slugify_article_on_update")]    
    article: crate::models::article::UpdateArticleRequest,
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

fn slugify_article_on_update<'de, D>(deserializer: D) 
    -> std::result::Result<crate::models::article::UpdateArticleRequest, D::Error>
where
    D: serde::Deserializer<'de>, {
    use slugify::slugify;
    let mut req: crate::models::article::UpdateArticleRequest = serde::Deserialize::deserialize(deserializer)?;
    req.slug_from_title = req.title.as_ref().and_then(|title| Some(slugify!(title)));
    Ok(req)
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
