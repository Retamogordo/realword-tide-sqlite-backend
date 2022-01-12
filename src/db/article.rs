use sqlx::{SqliteConnection, Pool};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, Sqlite};
use crate::{models::article, filters, errors};

pub(crate) async fn create_article(conn: &Pool<Sqlite>,
    author_name: &str,
    article: &article::CreateArticleRequest,
) -> Result<article::ArticleResponse, errors::RegistrationError>  {
    sqlx::query(
        "INSERT INTO articles (author, slug, title, description, body, tagList, createdAt, updatedAt)
        VALUES( ?,	?, ?, ?, ?, ?, datetime('now'), datetime('now'));
        ")
    .bind(&author_name)
    .bind(&article.slug)
    .bind(&article.title)
    .bind(&article.description)
    .bind(&article.body)
    .bind(
        &article.tag_list.as_ref()
            .and_then(|tags| 
                Some( tags
                    .iter()
                    .fold("".to_string(), |s, tag| format!("{}{},", s, tag) ) )
            )
    )
    .execute(conn)    
    .await?;

    let article = get_one(conn, 
//        crate::filters::ArticleFilterEnum::BySlug(&article.slug)
        crate::filters::ArticleFilterBySlug { slug: &article.slug },
    ).await?;
    Ok(article)
}

fn get_article_clause<F: crate::filters::ArticleFilter>(
//    filter: &crate::filters::ArticleFilterEnum<'_>, 
    filter: &F, 
    order_by: &crate::filters::OrderByFilter,
    limit_offset: &crate::filters::LimitOffsetFilter,
) -> String  {
    format!(" \
        SELECT *, (favoritesCount>0) as favorited FROM \
            (SELECT articles.id as id, slug, title, body, description, tagList, createdAt, updatedAt, author,	COUNT(favorite_articles.id) as favoritesCount FROM articles \
            LEFT JOIN favorite_articles ON articles.id = favorite_articles.id WHERE {} \
            {} {}) \
        WHERE id IS NOT NULL", 
    filter, order_by, limit_offset)
}

/*
pub(crate) async fn get_article<F: filters::ArticleFilter>(conn: &Pool<Sqlite>,
//    filter: article::filters::ArticleFilterEnum<'_>
    filter: F
) -> Result<article::ArticleResponse, errors::RegistrationError>  {

    let statement = get_article_clause(&filter, 
        &crate::filters::OrderByFilter::Descending("updatedAt"), 
        &crate::filters::LimitOffsetFilter::default());

    let article = sqlx::query_as::<_, article::Article>(
        &statement
    )
    .fetch_optional(conn)    
    .await?;

    if let Some(article) = article {
        let author = super::user::get_profile(conn, &article.author).await;
        Ok(article::ArticleResponse { article, author })    
    } else {
        Err(errors::RegistrationError::NoArticleFound)
    }
}
*/
pub(crate) async fn get_one<F: crate::filters::ArticleFilter>(conn: &Pool<Sqlite>,
    filter: F,
) -> Result<article::ArticleResponse, errors::RegistrationError>  {
    let limit_offset = crate::filters::LimitOffsetFilter { 
        limit: Some(1), 
        offset: None 
    };

    let articles = get_articles(conn, filter, crate::filters::OrderByFilter::default(), limit_offset).await?;
    if let Some(article) = articles.into_iter().next() {
        Ok(article)    
    } else {
        Err(errors::RegistrationError::NoArticleFound)
    }
}

pub(crate) async fn get_all<F: crate::filters::ArticleFilter>(conn: &Pool<Sqlite>,
    filter: F,
    order_by: crate::filters::OrderByFilter<'_>,
    limit_offset: crate::filters::LimitOffsetFilter
) -> Result<article::MultipleArticleResponse, errors::RegistrationError>  {

    Ok(article::MultipleArticleResponse::from_articles( 
        get_articles(conn, filter, order_by, limit_offset).await?)
    )    
}

async fn get_articles<F: crate::filters::ArticleFilter>(conn: &Pool<Sqlite>,
 //   filter: crate::filters::ArticleFilterEnum<'_>,
    filter: F,
    order_by: crate::filters::OrderByFilter<'_>,
    limit_offset: crate::filters::LimitOffsetFilter
) -> Result<Vec::<article::ArticleResponse>, errors::RegistrationError>  {
  
    let statement = get_article_clause(&filter, &order_by, &limit_offset);

    let articles = sqlx::query_as::<_, article::Article>(
        &statement
    )
//    .fetch_optional(conn)    
    .fetch_all(conn)    
    .await?;

    let mut multiple_articles = Vec::<article::ArticleResponse>::with_capacity(articles.len());

    for article in articles {
        let author = super::user::get_profile(conn, &article.author).await;
        multiple_articles.push( article::ArticleResponse { article, author } );
    }

//    if 0 != multiple_articles.len() {
//    Ok(article::MultipleArticleResponse::from_articles( multiple_articles ))    
    Ok(multiple_articles)    
    /*    } else { 
        Err(errors::RegistrationError::NoArticleFound)
    }*/
}

pub(crate) async fn update_article(conn: &Pool<Sqlite>,
        filter: crate::filters::UpdateArticleFilter<'_>
) -> Result<article::ArticleResponse, errors::RegistrationError>  {

    let statement = format!("UPDATE articles SET {}", filter);
    sqlx::query(&statement)
        .execute(conn)    
        .await?;

    let updated_slug = filter.updated_slug();
    get_one(conn, filters::ArticleFilterBySlug { slug: updated_slug })
        .await
}
    
pub(crate) async fn favorite_article<F: crate::filters::ArticleFilter>(conn: &Pool<Sqlite>,
    filter: F,
    username: &str,
) -> Result<article::ArticleResponse, errors::RegistrationError>  {

    let statement = format!("\
        INSERT INTO favorite_articles (id, username) VALUES ( \
            (SELECT id FROM articles WHERE {}), '{}') \
            ON CONFLICT DO NOTHING; \
        ", filter.to_string(), username);
    
    sqlx::query(
        &statement
    )
    .execute(conn)
    .await?;        

    get_one(conn, filter).await
}

pub(crate) async fn unfavorite_article<F: filters::ArticleFilter>(conn: &Pool<Sqlite>,
    filter: F,
    username: &str,
) -> Result<article::ArticleResponse, errors::RegistrationError>  {

    let statement = format!("\
        DELETE FROM favorite_articles WHERE favorite_articles.id= \
            (SELECT id FROM articles WHERE {}) \
        ", filter.to_string());
    
    sqlx::query(
        &statement
    )
    .execute(conn)
    .await?;        

    get_one(conn, filter).await
}

pub(crate) async fn get_comments(conn: &Pool<Sqlite>,
    filter: filters::CommentFilterByValues<'_>,
    order_by: filters::OrderByFilter<'_>,
    limit_filter: crate::filters::LimitOffsetFilter,
) -> Result<article::MultipleCommentResponse, errors::RegistrationError>  {

    let statement = format!("SELECT * FROM comments WHERE {} {} {}", filter, order_by, limit_filter);

    let comments = sqlx::query_as::<_, article::Comment>(
        &statement
    )
    .fetch_all(conn)  
    .await?;
    
    let mut multiple_comments = Vec::<article::CommentResponse>::with_capacity(comments.len());

    for comment in comments {
        let author = super::user::get_profile(conn, &comment.author).await;
        multiple_comments.push( article::CommentResponse { comment, author } );
    }
    Ok(article::MultipleCommentResponse::from_comments( multiple_comments ))    
}
    
pub(crate) async fn add_comment(conn: &Pool<Sqlite>,
    filter: crate::filters::ArticleFilterBySlug<'_>,
    author: &str,
    comment: &article::AddCommentRequest,
) -> Result<article::CommentResponse, errors::RegistrationError>  {
//) -> Result<(), errors::RegistrationError>  {
    let statement = format!("INSERT INTO comments (author, body, createdAt, updatedAt, article_id) VALUES( '{}','{}', datetime('now'), datetime('now'), (SELECT id FROM articles WHERE {} LIMIT 1));", author, comment.body, filter);
    sqlx::query(&statement)
//    .bind(&author)
//    .bind(&comment.body)
//    .bind(&filter.to_string())
    .execute(conn)    
    .await?;

    let comment_filter = filters::CommentFilterByValues {
        author: Some(author),
        article_slug: Some(filter.slug),
    };
    let order_by = crate::filters::OrderByFilter::Descending("id");
    let limit_filter = crate::filters::LimitOffsetFilter { limit: Some(1), offset: None };

    let comments_response = get_comments(conn, comment_filter, order_by, limit_filter).await?;
    if let Some(comment) = comments_response.comments.into_iter().next() {
        Ok(comment)
    } else {
        Err(errors::RegistrationError::NoCommentFound)
    }
}

pub(crate) async fn get_tags(conn: &Pool<Sqlite>,
) -> Result<article::TagList, errors::RegistrationError>  {

    let statement = format!("SELECT tagList FROM articles");

    let all_tags: Vec<String> = sqlx::query_scalar(
        &statement
    )
    .fetch_all(conn)  
    .await?;

    let mut all_tags_hash_set 
        = std::collections::HashSet::<&str>::with_capacity(200*all_tags.len());

    for tags_delimited in &all_tags {
        let tag_hash_set = 
            tags_delimited.split(",")
                .map(|tag| tag.trim())
                .filter(|tag| *tag != "")
                .collect::<std::collections::HashSet<&str>>();

        all_tags_hash_set.extend(tag_hash_set);    
    }

    let mut tags: Vec<String> = all_tags_hash_set
        .into_iter()
        .map(|tag| tag.to_string() )
        .collect();

    tags.sort_by_key(|tag| tag.to_lowercase());

    Ok(article::TagList {tags})
}

