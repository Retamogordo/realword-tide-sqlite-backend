use sqlx::Connection;
use sqlx::{SqliteConnection, Pool};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, Sqlite};
use std::str::FromStr;
use sqlx::Row;

#[derive(sqlx::FromRow)]
pub(crate) struct UserDB {
    email: String,    
    username: String,    
    password: String,
}

pub async fn connect(database_url_prefix: &str, database_url_path: &str, 
    database_file: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
//    let conn = SqliteConnection::connect("sqlite::memory:").await?;

//    let database_url = "sqlite:///home/yury/sqlite/my_test.db";
    match async_std::fs::create_dir(database_url_path).await {
        Ok(()) => Ok(()),
        Err(err) if async_std::io::ErrorKind::AlreadyExists == err.kind() => Ok(()),
        err @ _ => err, 
    }?;

    let database_url = format!("{}{}", database_url_prefix, database_url_path);

    let connection_options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .filename(database_file);
//        .journal_mode(SqliteJournalMode::Wal)
//        .synchronous(SqliteSynchronous::Normal)
//        .busy_timeout(pool_timeout);

    let sqlite_pool = SqlitePoolOptions::new()
//        .max_connections(pool_max_connections)
//        .connect_timeout(pool_timeout)
        .connect_with(connection_options)
        .await?;
    
 //   sqlx::migrate!("../../../sqlite").run(&sqlite_pool).await?;

    sqlx::query("
        DROP TABLE IF EXISTS users;
        CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL
        );
    ")
    .execute(&sqlite_pool)    
    .await?;
    
    sqlx::query("
        DROP TABLE IF EXISTS profiles;
        CREATE TABLE IF NOT EXISTS profiles (
            username TEXT NOT NULL,
            bio TEXT,
            image TEXT,
            FOREIGN KEY (username)
               REFERENCES users (username) 
               ON DELETE CASCADE
               ON UPDATE CASCADE
        );
    ")
    .execute(&sqlite_pool)    
    .await?;

    sqlx::query("
        DROP TABLE IF EXISTS followers;
        CREATE TABLE IF NOT EXISTS followers (
            follower_name TEXT NOT NULL,
            celeb_name TEXT NOT NULL,
            FOREIGN KEY (celeb_name)
                REFERENCES users (username) 
                ON DELETE CASCADE
                ON UPDATE CASCADE,
            FOREIGN KEY (follower_name)
                REFERENCES users (username) 
                ON DELETE CASCADE
                ON UPDATE CASCADE
            CONSTRAINT Pair UNIQUE (follower_name,celeb_name)
        );
    ")
    .execute(&sqlite_pool)    
    .await?;

    sqlx::query("
        DROP TABLE IF EXISTS articles;
        CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            slug TEXT,
            title TEXT NOT NULL,
            description TEXT,
            body TEXT NOT NULL,
            tagList TEXT,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL,
            author TEXT NOT NULL   
        );
    ")
    .execute(&sqlite_pool)    
    .await?;
    
    sqlx::query("
        DROP TABLE IF EXISTS favorite_articles;
        CREATE TABLE IF NOT EXISTS favorite_articles (
            id INTEGER NOT NULL,
            username TEXT NOT NULL,
            FOREIGN KEY (id)
                REFERENCES articles (id) 
                ON DELETE CASCADE
            FOREIGN KEY (username)
                REFERENCES users (username) 
                ON DELETE CASCADE
                ON UPDATE CASCADE
            CONSTRAINT Pair UNIQUE (id, username)
        );
    ")
    .execute(&sqlite_pool)    
    .await?;

    sqlx::query("
        DROP TABLE IF EXISTS comments;
        CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            article_id INTEGER NOT NULL,
            body TEXT NOT NULL,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL,
            author TEXT NOT NULL,   
        FOREIGN KEY (article_id)
            REFERENCES articles (id) 
            ON DELETE CASCADE
        FOREIGN KEY (author)
            REFERENCES users (username) 
            ON DELETE CASCADE
            ON UPDATE CASCADE
        );
    ")
    .execute(&sqlite_pool)    
    .await?;

    Ok(sqlite_pool)
}

pub(crate) async fn register_user(conn: &Pool<Sqlite>,
    user: &crate::UserReg,
) -> Result<(), crate::errors::RegistrationError>  {
    sqlx::query(
            "INSERT INTO users (username, email, password)
            VALUES( ?,	?, ?);\n
            INSERT INTO profiles (username)
            VALUES( ?);
            ")
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.username)
        .execute(conn)    
        .await?;
    Ok(())
}

pub(crate) async fn get_user_by_email(conn: &Pool<Sqlite>,
    email: &str,
) -> Result<crate::User, crate::errors::RegistrationError>  {

    let user: crate::User = sqlx::query_as::<_, crate::User>(
        "SELECT *, NULL as `token` FROM users LEFT JOIN profiles ON users.username = profiles.username WHERE email = ?;")
//            "SELECT *, NULL as `token` FROM users LEFT JOIN profiles ON users.username = profiles.username WHERE users.username = ?;")
        .bind(email)
        .fetch_optional(conn)   
        .await?
        .ok_or(crate::errors::RegistrationError::NoUserFound(email.to_string()))?;
    Ok(user)
}

pub(crate) async fn get_user_by_username(conn: &Pool<Sqlite>,
    username: &str,
) -> Result<crate::User, crate::errors::RegistrationError>  {

    let user: crate::User = sqlx::query_as::<_, crate::User>(
        "SELECT *, NULL as `token` FROM users LEFT JOIN profiles ON users.username = profiles.username WHERE users.username = ?;")
//            "SELECT *, NULL as `token` FROM users LEFT JOIN profiles ON users.username = profiles.username WHERE users.username = ?;")
        .bind(username)
        .fetch_optional(conn)   
        .await?
        .ok_or(crate::errors::RegistrationError::NoUserFound(username.to_string()))?;
    Ok(user)
}

pub(crate) async fn update_user(conn: &Pool<Sqlite>,
    username: &str,
    user: &crate::UserUpdate,
) -> Result<crate::User, crate::errors::RegistrationError>  {
    // use "dummy" set username=username for case there is nothing to update,
    // probably there is better way to perform this "empty update"
    let statement = "UPDATE users SET username=username, "; 
    let mut s = format!("{}", statement);
 //  let mut email_changed = false;

    let new_username = if let Some(new_username) = user.username.as_ref() {
        s = format!("{} username = '{}',", s, new_username);
        new_username
    } else { username };

    if let Some(email) = user.email.as_ref() {
        s = format!("{} email = '{}',", s, email);
 //       email_changed = true;
    }
    s = format!("{} WHERE username = '{}';", s.split_at(s.len()-1).0, username);
    
    // use "dummy" set bio=bio for case there is nothing to update
    // probably there is better way to perform this "empty update"
    s = format!("{} UPDATE profiles SET bio=bio,", s); 
    if let Some(bio) = user.bio.as_ref() {
        s = format!("{} bio = '{}',", s, bio);
    }
    if let Some(image) = user.username.as_ref() {
        s = format!("{} image = '{}',", s, image);
    }
    s = format!("{} WHERE username = '{}';", s.split_at(s.len()-1).0, new_username);
    
    sqlx::query(&s)
        .execute(conn)    
        .await?;

    get_user_by_username(conn, new_username).await
}

pub(crate) async fn get_profile(conn: &Pool<Sqlite>,
    username: &str,
) -> Option<crate::Profile> {
//-> Result<crate::Profile, crate::errors::RegistrationError>  {

    let profile = sqlx::query_as::<_, crate::Profile>(
            &format!("SELECT *, 
                (SELECT COUNT(*)>0 FROM followers 
                    WHERE celeb_name = '{}'
                    ) AS following
            FROM profiles 
            INNER JOIN users ON profiles.username = users.username 
            WHERE profiles.username = '{}';
    ", username, username))
//        .bind(username)
//        .bind(username)
        .fetch_optional(conn)   
        .await
        .unwrap_or(None);

    profile
//        .ok_or(crate::errors::RegistrationError::NoUserFound(username.to_string()))?;
//    Ok(profile)
}

pub(crate) async fn follow(conn: &Pool<Sqlite>,
    follower_name: &str,
    celeb_name: &str,
) -> Result<crate::Profile, crate::errors::RegistrationError>  {
    
    sqlx::query("INSERT INTO followers (follower_name, celeb_name)
        VALUES( ?,?) ON CONFLICT DO NOTHING;")
        .bind(follower_name)
        .bind(celeb_name)
        .execute(conn)    
        .await?;

    get_profile(conn, celeb_name)
        .await
        .ok_or(crate::errors::RegistrationError::NoUserFound(celeb_name.to_string()))
}
pub(crate) async fn unfollow(conn: &Pool<Sqlite>,
    follower_name: &str,
    celeb_name: &str,
) -> Result<crate::Profile, crate::errors::RegistrationError>  {

    let statement = format!("DELETE FROM followers WHERE follower_name='{}' AND celeb_name='{}';", follower_name, celeb_name);
    sqlx::query(&statement)
//        .bind(follower_name)
//        .bind(celeb_name)
        .execute(conn)    
        .await?;

    get_profile(conn, celeb_name)
        .await
        .ok_or(crate::errors::RegistrationError::NoUserFound(celeb_name.to_string()))

}

pub(crate) async fn create_article(conn: &Pool<Sqlite>,
    author_name: &str,
    article: &crate::CreateArticleRequest,
) -> Result<crate::ArticleResponse, crate::errors::RegistrationError>  {
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

    let article = get_article(conn, 
//        crate::ArticleFilterEnum::BySlug(&article.slug)
        crate::ArticleFilterBySlug { slug: &article.slug }
    ).await?;
    Ok(article)
}

fn get_article_clause<F: crate::ArticleFilter>(
//    filter: &crate::ArticleFilterEnum<'_>, 
    filter: &F, 
    order_by: &crate::OrderByFilter,
    limit_offset: &crate::LimitOffsetFilter,
) -> String  {
    format!(" \
        SELECT *, (favoritesCount>0) as favorited FROM \
            (SELECT articles.id as id, slug, title, body, description, tagList, createdAt, updatedAt, author,	COUNT(favorite_articles.id) as favoritesCount FROM articles \
            LEFT JOIN favorite_articles ON articles.id = favorite_articles.id WHERE {} \
            {}) \
            {}
        WHERE id IS NOT NULL", 
    filter, order_by, limit_offset)
}

pub(crate) async fn get_article<F: crate::ArticleFilter>(conn: &Pool<Sqlite>,
//    filter: crate::ArticleFilterEnum<'_>
    filter: F
) -> Result<crate::ArticleResponse, crate::errors::RegistrationError>  {

    let statement = get_article_clause(&filter, 
        &crate::OrderByFilter::Descending("updatedAt"), 
        &crate::LimitOffsetFilter::default());

    let article = sqlx::query_as::<_, crate::Article>(
        &statement
    )
    .fetch_optional(conn)    
    .await?;

    if let Some(article) = article {
        let author = get_profile(conn, &article.author).await;
        Ok(crate::ArticleResponse { article, author })    
    } else {
        Err(crate::errors::RegistrationError::NoArticleFound)
    }
}

pub(crate) async fn get_articles<F: crate::ArticleFilter>(conn: &Pool<Sqlite>,
 //   filter: crate::ArticleFilterEnum<'_>,
    filter: F,
    order_by: crate::OrderByFilter<'_>,
    limit_offset: crate::LimitOffsetFilter
) -> Result<crate::MultipleArticleResponse, crate::errors::RegistrationError>  {
  
    let statement = get_article_clause(&filter, &order_by, &limit_offset);

    let articles = sqlx::query_as::<_, crate::Article>(
        &statement
    )
//    .fetch_optional(conn)    
    .fetch_all(conn)    
    .await?;

    let mut multiple_articles = Vec::<crate::ArticleResponse>::with_capacity(articles.len());

    for article in articles {
        let author = get_profile(conn, &article.author).await;
        multiple_articles.push( crate::ArticleResponse { article, author } );
    }

//    if 0 != multiple_articles.len() {
        Ok(crate::MultipleArticleResponse::from_articles( multiple_articles ))    
/*    } else { 
        Err(crate::errors::RegistrationError::NoArticleFound)
    }*/
}

pub(crate) async fn update_article(conn: &Pool<Sqlite>,
        filter: crate::UpdateArticleFilter<'_>
) -> Result<crate::ArticleResponse, crate::errors::RegistrationError>  {

    let statement = format!("UPDATE articles SET {}", filter.to_string());
    sqlx::query(&statement)
        .execute(conn)    
        .await?;

    let updated_slug = filter.updated_slug();
    get_article(conn, crate::ArticleFilterBySlug { slug: updated_slug })
        .await
}
    
pub(crate) async fn favorite_article<F: crate::ArticleFilter>(conn: &Pool<Sqlite>,
    filter: F,
    username: &str,
) -> Result<crate::ArticleResponse, crate::errors::RegistrationError>  {

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

    get_article(conn, filter).await
}

pub(crate) async fn unfavorite_article<F: crate::ArticleFilter>(conn: &Pool<Sqlite>,
    filter: F,
    username: &str,
) -> Result<crate::ArticleResponse, crate::errors::RegistrationError>  {

    let statement = format!("\
        DELETE FROM favorite_articles WHERE favorite_articles.id= \
            (SELECT id FROM articles WHERE {}) \
        ", filter.to_string());
    
    sqlx::query(
        &statement
    )
    .execute(conn)
    .await?;        

    get_article(conn, filter).await
}

pub(crate) async fn get_comments(conn: &Pool<Sqlite>,
    filter: crate::CommentFilterByValues<'_>,
    order_by: crate::OrderByFilter<'_>,
    limit_filter: crate::LimitOffsetFilter,
) -> Result<crate::MultipleCommentResponse, crate::errors::RegistrationError>  {

    let statement = format!("SELECT * FROM comments WHERE {} {} {}", filter, order_by, limit_filter);

    let comments = sqlx::query_as::<_, crate::Comment>(
        &statement
    )
    .fetch_all(conn)  
    .await?;
    
    let mut multiple_comments = Vec::<crate::CommentResponse>::with_capacity(comments.len());

    for comment in comments {
        let author = get_profile(conn, &comment.author).await;
        multiple_comments.push( crate::CommentResponse { comment, author } );
    }
    Ok(crate::MultipleCommentResponse::from_comments( multiple_comments ))    

/*    if let Some(comment) = comment {
        let author = get_profile(conn, &comment.author).await;
        Ok(crate::CommentResponse { comment, author })    
    } else {
        Err(crate::errors::RegistrationError::NoCommentFound)
    }*/
}
    
pub(crate) async fn add_comment(conn: &Pool<Sqlite>,
    filter: crate::ArticleFilterBySlug<'_>,
    author: &str,
    comment: &crate::AddCommentRequest,
) -> Result<crate::CommentResponse, crate::errors::RegistrationError>  {
//) -> Result<(), crate::errors::RegistrationError>  {
    let statement = format!("INSERT INTO comments (author, body, createdAt, updatedAt, article_id) VALUES( '{}','{}', datetime('now'), datetime('now'), (SELECT id FROM articles WHERE {} LIMIT 1));", author, comment.body, filter);
    sqlx::query(&statement)
//    .bind(&author)
//    .bind(&comment.body)
//    .bind(&filter.to_string())
    .execute(conn)    
    .await?;

    let comment_filter = crate::CommentFilterByValues {
        author: Some(author),
        article_slug: Some(filter.slug),
    };
    let order_by = crate::OrderByFilter::Descending("id");
    let limit_filter = crate::LimitOffsetFilter { limit: Some(1), offset: None };

    let comments_response = get_comments(conn, comment_filter, order_by, limit_filter).await?;
    if let Some(comment) = comments_response.comments.into_iter().next() {
        Ok(comment)
    } else {
        Err(crate::errors::RegistrationError::NoCommentFound)
    }
}

pub(crate) async fn get_tags(conn: &Pool<Sqlite>,
) -> Result<crate::TagList, crate::errors::RegistrationError>  {

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

    Ok(crate::TagList {tags})
}

