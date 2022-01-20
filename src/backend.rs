use sqlx::sqlite::{SqlitePool};

use crate::{config::Config, 
    models::{user::*, article::*}, 
    requests,
    db, auth, filters, errors::BackendError
};

use crate::requests::IntoAuthenticatedRequest;

#[derive(Clone, Debug)]
pub struct Server {
    config: Config,
    pub(crate) conn: Option<SqlitePool>,
}

impl Server {
    pub fn with_config(config: Config) -> Self {
        Self { 
            config,
            conn: None,
        }
    }
 
    pub async fn connect(&mut self) -> Result<(), sqlx::Error> {
        self.conn = Some(db::connect(&self.config).await?);
        Ok(())
    }

    pub fn secret(&self) -> &[u8] {
        self.config.secret.as_bytes()
    }

    pub async fn register_user(&self, user_reg: requests::user::UserReg) -> Result<User, BackendError> {

        let user_to_reg = User::try_from(user_reg)?;

        let mut user = db::user::register_user(self.conn.as_ref().unwrap(), &user_to_reg).await?;
        // login by creating token
        user.token = Some(auth::Auth::create_token(&user, self.secret())?);
        Ok(user)
    }

    pub async fn login_user(&self, login_req: requests::user::LoginRequest) -> Result<User, BackendError> {
        let user_by = filters::UserFilter::default().email(&login_req.email);

        let mut user = db::user::get_user(self.conn.as_ref().unwrap(), user_by)
            .await
            .map_err(|err| 
                match err {
                    BackendError::NoUserFound(_) =>
                        BackendError::IncorrectUsernameOrPassword(login_req.email.clone()),
                    err @ _ => err,
                }
            )?;

        user.verify(&login_req.password)?;

        user.token = Some(auth::Auth::create_token(&user, self.secret())?);

        Ok(user)
    }

    pub async fn user_by_token(&self, token: &str) -> Result<User, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;
        let filter = filters::UserFilter::default().username(&claims.username);

        let mut user = db::user::get_user(self.conn.as_ref().unwrap(), filter).await?;
        user.token = Some(token.to_string());
        Ok(user)
    }

    pub async fn profile(&self, username: &str) -> Result<Profile, BackendError> {
        db::user::get_profile(self.conn.as_ref().unwrap(), username)
            .await
            .ok_or(BackendError::NoUserFound(username.to_string()))
    }

    pub async fn update_user(&self, token: &str, update_user_req: requests::user::UserUpdateRequest) -> Result<User, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;
        let filter = filters::UpdateUserFilter::default().username(&claims.username);
        let update_user = UserUpdate::from(&update_user_req);

        let mut user = db::user::update_user(self.conn.as_ref().unwrap(), &update_user, filter).await?;
        user.token = Some(token.to_string());
        Ok(user)
    }

    pub async fn follow(&self, token: &str, celeb_name: &str) -> Result<Profile, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;

        db::user::follow(self.conn.as_ref().unwrap(), &claims.username, &celeb_name).await
    }   

    pub async fn unfollow(&self, token: &str, celeb_name: &str) -> Result<Profile, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;

        db::user::unfollow(self.conn.as_ref().unwrap(), &claims.username, &celeb_name).await
    }

    pub async fn create_article(&self, 
        token: &str, 
        article_request: requests::article::CreateArticleRequest) -> Result<ArticleResponse, BackendError> {
        
        let create_req_auth = article_request.authenticate(token, self.secret())?; 
        let article = Article::from(create_req_auth);

        db::article::create_article(self.conn.as_ref().unwrap(), article).await
    }

    pub async fn get_article_by_slug(&self, slug: &str) -> Result<ArticleResponse, BackendError> {
        let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());

        db::article::get_one(self.conn.as_ref().unwrap(), filter).await
    }

    pub async fn get_articles(&self, 
        article_by: filters::ArticleFilterByValues,
        order_by: filters::OrderByFilter<'_>,
        limit_offset: filters::LimitOffsetFilter
    ) -> Result<MultipleArticleResponse, BackendError> {

        let articles = db::article::get_all(self.conn.as_ref().unwrap(), 
            article_by, 
            order_by, 
            limit_offset)
        .await?;
        Ok(MultipleArticleResponse::from_articles(articles))
    }

    pub async fn update_article(&self, 
        token: &str, 
        update_article_req: requests::article::UpdateArticleRequest<'_>, 
    ) -> Result<ArticleResponse, BackendError> {
    
    let update_req_auth = update_article_req.authenticate(token, self.secret())?;
    let update_by = filters::UpdateArticleFilter::from(&update_req_auth);
    let slug = update_by.slug;

    let res = match db::article::update_article(self.conn.as_ref().unwrap(), 
                                                &update_req_auth.article_request.article, 
                                                update_by).await {
        Ok(article_response) => {
            Ok(article_response)
        },
        Err(err) => match err {
            // successful update returns the updated article, otherwise
            // NoArticleFound error is returned
            // if optimistic update fails, try to verify if this happened
            // because user is not authorized to do so
            BackendError::NoArticleFound => {
                let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());
                if let Ok(article_response) = db::article::get_one(self.conn.as_ref().unwrap(), filter).await {
                    auth::Auth::authorize(token, self.secret(), &article_response.article.author)?;

                    Err(BackendError::UnexpectedError(
                        " could not update article despite user was authorized to do so.".to_string())
                    )
                } else {
                    Err(BackendError::NoArticleFound)
                }
            },
            err @ _ =>  Err(err),
            }
        };
        res
    }

    pub async fn delete_article(&self, 
        token: &str, 
        slug: &str,
    ) -> Result<(), BackendError> {
        
        let delete_req = requests::article::DeleteArticleRequest {
            slug
        };
        let delete_req_auth = delete_req.authenticate(token, self.secret())?;
        let delete_by = filters::UpdateArticleFilter::from(&delete_req_auth);

        let query_res = db::article::delete_article(self.conn.as_ref().unwrap(), 
                                                    delete_by).await?;
        if 0 == query_res.rows_affected() {
                                            
            let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());
            if let Ok(article_response) = db::article::get_one(self.conn.as_ref().unwrap(), filter).await {

                auth::Auth::authorize(token, self.secret(), &article_response.article.author)?;

                Err(BackendError::UnexpectedError(
                    "could not delete article despite user was authorized to do so.".to_string())
                )
            } else {
                Err(BackendError::NoArticleFound)
            }
        } else {
            Ok(())
        }
    }

    pub async fn favorite_article(&self, token: &str, slug: &str) -> Result<ArticleResponse, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;

        db::article::favorite_article(self.conn.as_ref().unwrap(), slug, &claims.username).await
    }

    pub async fn unfavorite_article(&self, token: &str, slug: &str) -> Result<ArticleResponse, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;

        db::article::unfavorite_article(self.conn.as_ref().unwrap(), slug, &claims.username).await
    }

    pub async fn feed_articles(&self, 
        token: &str,
        limit_offset: filters::LimitOffsetFilter
    ) -> Result<MultipleArticleResponse, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;

        let filter = filters::ArticleFilterFeed { follower: &claims.username };
        let order_by = filters::OrderByFilter::Descending("updatedAt");
        
        let articles = db::article::get_all(self.conn.as_ref().unwrap(), 
            filter, 
            order_by, 
            limit_offset)
        .await?;

        Ok(MultipleArticleResponse::from_articles(articles))
    }

    pub async fn add_comment(&self, 
        token: &str, 
        add_comment_req: requests::article::AddCommentRequest<'_>
    ) -> Result<CommentResponse, BackendError> {

        let add_req_auth = &add_comment_req.authenticate(token, self.secret())?;

        let comment_filter = filters::CommentFilterByValues::default()
            .article_slug(add_req_auth.article_request.article_slug);
        let comment_author = &add_req_auth.author;
        let comment_body = &add_req_auth.article_request.body;

        db::article::add_comment(self.conn.as_ref().unwrap(), comment_filter, comment_author,
        comment_body).await
    }

    pub async fn delete_comment(&self, 
        token: &str, 
        delete_req: requests::article::DeleteCommentRequest<'_>,
    ) -> Result<(), BackendError> {
 
        let id = delete_req.id;

        let delete_req_auth = &delete_req.authenticate(token, self.secret())?;
        let filter = filters::CommentFilterByValues::from(delete_req_auth);

        let query_res = db::article::delete_comments(self.conn.as_ref().unwrap(), filter).await?;
        // if no comment has been deleted, check if user is authorized to do so    
        if 0 == query_res.rows_affected() {

            let filter = filters::CommentFilterByValues::default().id(id);

            let comments = db::article::get_comments(self.conn.as_ref().unwrap(), 
                                                    filter, 
                                                    filters::OrderByFilter::default(), 
                                                    filters::LimitOffsetFilter::default().limit(1))
                .await; 

            match comments {
                Ok(comments) =>
                    if let Some(comment) = comments.iter().next() {
    
                        auth::Auth::authorize(token, self.secret(), &comment.comment.author)?;
                        
                        Err(BackendError::UnexpectedError(
                            "could not delete comment despite user has been authorized to, probably due to a bug".to_string()))
    
                    } else {
                        Err(BackendError::NoCommentFound(id))
                    },
                Err(err) => Err(err)
            }
        } else {
            Ok(())
        }
    }

    pub async fn get_comments(&self, slug: &str) -> Result<MultipleCommentResponse, BackendError> {
        let filter = filters::CommentFilterByValues::default().article_slug(slug);
        let order_by = filters::OrderByFilter::Descending("id");
        let limit_offset: filters::LimitOffsetFilter = filters::LimitOffsetFilter::default();
    
        let comments = db::article::get_comments(
            self.conn.as_ref().unwrap(), filter, order_by, limit_offset)
            .await?;
        Ok(MultipleCommentResponse { comments } )
    }

    pub async fn get_tags(&self) -> Result<TagList, BackendError> {
        db::article::get_tags(self.conn.as_ref().unwrap()).await 
    }   
}
