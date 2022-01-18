use tide::prelude::*;
use crate::models::article::*;

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

pub(crate) struct CreateArticleRequestAuthenicated<'a> {
    pub article: &'a CreateArticleRequest,
    pub author: &'a str,
}



#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub(crate) struct UpdateArticleRequest {
    #[serde(deserialize_with = "slugify_article_on_update")]    
    pub article: UpdateArticle,
}

fn slugify_article_on_update<'de, D>(deserializer: D) 
    -> std::result::Result<UpdateArticle, D::Error>
where
    D: serde::Deserializer<'de>, {
    use slugify::slugify;
    let mut article: UpdateArticle = serde::Deserialize::deserialize(deserializer)?;
    article.slug_from_title = article.title.as_ref().and_then(|title| Some(slugify!(title)));
    Ok(article)
}

