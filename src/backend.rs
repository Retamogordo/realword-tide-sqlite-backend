use sqlx::sqlite::{SqlitePool};
use validator::{Validate};

use crate::{config::Config, models::{user::*, article::*}, errors::*, db, auth, filters};
/*
#[async_trait]
pub trait Service {
    type ResultType;

    async fn execute(&self) -> Result<Self::ResultType, crate::errors::BackendError>;
    fn auth(self, token: &str) -> Self;
}

pub enum Services {
    RegisterUser(UserReg),
    Login,
    UpdateUser,

}
*/
#[derive(Clone, Debug)]
pub struct Server {
    config: Config,
    pub(crate) conn: Option<SqlitePool>,
//    secret: Option<&'static [u8]>,
}

impl Server {
    pub fn with_config(config: Config) -> Self {
        Self { 
            config,
            conn: None,
        }
    }
 
    pub async fn connect(&mut self) -> Result<(), sqlx::Error> {

        self.conn = Some(crate::db::connect(&self.config)
            .await?);
        Ok(())
//            .expect("failed to connect to sqlite database. ")
    }
 //   pub fn with_db_conn(conn: SqlitePool) -> Self {
 //       Self { conn, secret: None }
 //   }

    pub fn secret(&self) -> &[u8] {
        self.config.secret.as_bytes()
    }

    pub async fn register_user(&self, user_reg: UserReg) -> Result<User, BackendError> {
        user_reg.validate()?;

        db::user::register_user(self.conn.as_ref().unwrap(), &user_reg).await?;
    
        let mut user: User = user_reg.into();
        user.token = Some(auth::Auth::create_token(&user, self.secret())?);
        Ok(user)
    }

    pub async fn login_user(&self, user_by: filters::UserFilter) -> Result<User, BackendError> {
        let mut user = db::user::get_user(self.conn.as_ref().unwrap(), user_by).await?;

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
            .ok_or(crate::errors::BackendError::NoUserFound(username.to_string()))

/*        let res = match db::user::get_profile(self.conn.as_ref().unwrap(), username).await {
            Some(profile) => {
                Ok(profile)
            },
            None => 
                Err(crate::errors::BackendError::NoUserFound(username.to_string()))
        };
        res*/
    }

    pub async fn update_user(&self, token: &str, update_user: UserUpdate) -> Result<User, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;
        let filter = filters::UpdateUserFilter::default().username(&claims.username);

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

    pub async fn create_article(&self, token: &str, article: &CreateArticleRequest) -> Result<ArticleResponse, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;

        db::article::create_article(self.conn.as_ref().unwrap(), &claims.username, article).await
    }

    pub async fn get_article(&self, article_by: filters::ArticleFilterByValues) -> Result<ArticleResponse, BackendError> {
        db::article::get_one(self.conn.as_ref().unwrap(), article_by).await
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
        update_article: UpdateArticle, 
        slug: &str,
//        mut update_by: filters::UpdateArticleFilter<'_>,
    ) -> Result<ArticleResponse, BackendError> {
    
    let claims = auth::Auth::authenticate(token, self.secret())?;
    let update_by = filters::UpdateArticleFilter { 
        slug,
        author: &claims.username
    };

    let res = match db::article::update_article(self.conn.as_ref().unwrap(), 
                                                update_article, 
                                                update_by).await {
        Ok(article_response) => {
            Ok(article_response)
        },
        Err(err) => match err {
            // successful update returns the updated article, otherwise
            // NoArticleFound error is returned
            // if optimistic update fails, try to verify if this happened
            // because user is not authorized to do so
            crate::errors::BackendError::NoArticleFound => {
                let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());
                if let Ok(article_response) = db::article::get_one(self.conn.as_ref().unwrap(), filter).await {

 //                   let token = crate::utils::token_from_request(&req)?;
 //                   let secret = req.state().secret;
                    auth::Auth::authorize(token, self.secret(), &article_response.article.author)?;

                    Err(crate::errors::BackendError::UnexpectedError(
                        " could not update article despite user was authorized to do so.".to_string())
                    )
                } else {
                    Err(crate::errors::BackendError::NoArticleFound)
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
    //        mut update_by: filters::UpdateArticleFilter<'_>,
    ) -> Result<(), BackendError> {
        
        let claims = auth::Auth::authenticate(token, self.secret())?;
        let delete_by = filters::UpdateArticleFilter { 
            slug,
            author: &claims.username
        };

        let query_res = db::article::delete_article(self.conn.as_ref().unwrap(), 
                                                    delete_by).await?;
        if 0 == query_res.rows_affected() {
                                            

            let filter = filters::ArticleFilterByValues::default().slug(slug.to_string());
            if let Ok(article_response) = db::article::get_one(self.conn.as_ref().unwrap(), filter).await {

    //                   let token = crate::utils::token_from_request(&req)?;
    //                   let secret = req.state().secret;
                auth::Auth::authorize(token, self.secret(), &article_response.article.author)?;

                Err(crate::errors::BackendError::UnexpectedError(
                    " could not delete article despite user was authorized to do so.".to_string())
                )
            } else {
                Err(crate::errors::BackendError::NoArticleFound)
            }
        } else {
            Ok(())
        }
    }

    pub async fn favorite_article(&self, token: &str, favorite_by: filters::ArticleFilterByValues) -> Result<ArticleResponse, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;

        db::article::favorite_article(self.conn.as_ref().unwrap(), favorite_by, &claims.username).await
    }

    pub async fn unfavorite_article(&self, token: &str, unfavorite_by: filters::ArticleFilterByValues) -> Result<ArticleResponse, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;

        db::article::unfavorite_article(self.conn.as_ref().unwrap(), unfavorite_by, &claims.username).await
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
        article_filter: filters::ArticleFilterByValues,
        comment: AddCommentRequest
    ) -> Result<CommentResponse, BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;
        let author = &claims.username;

        db::article::add_comment(self.conn.as_ref().unwrap(), article_filter, author, comment).await
    }

    pub async fn get_comments(&self, 
//        token_opt: Option<&str>, 
        comments_filter: filters::CommentFilterByValues<'_>,
        order_by: filters::OrderByFilter<'_>,
        limit_offset: filters::LimitOffsetFilter,
    ) -> Result<MultipleCommentResponse, BackendError> {
//        let claims = auth::Auth::authenticate(token, self.secret())?;
/*        let tmp = token_opt
            .and_then(|token| 
                auth::Auth::authenticate(token, self.secret())
                    .ok()
                    .and_then(|claims| Some(claims.username))
            );
        // author is an Option, can be None
        filter.author = tmp.as_deref();
*/    
 //       let author = &claims.username;


        let comments = db::article::get_comments(self.conn.as_ref().unwrap(), comments_filter, order_by, limit_offset)
            .await?;
        Ok(MultipleCommentResponse { comments } )
    }

    pub async fn delete_comment(&self, 
        token: &str, 
        delete_by: filters::CommentFilterByValues<'_>,
    ) -> Result<(), BackendError> {
        let claims = auth::Auth::authenticate(token, self.secret())?;
        let author = &claims.username;

        let delete_by = delete_by.author(author);
        
        let id_opt = delete_by.id;

        let query_res = db::article::delete_comments(self.conn.as_ref().unwrap(), delete_by).await?;
        // if no comment has been deleted, check if user is authorized to do so    
        if 0 == query_res.rows_affected() {

            if let Some(id) = id_opt {
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
                Err(BackendError::UnexpectedError("tried to delete comment without id".to_string()))
            } 
    
        } else {
            Ok(())
        }
    }

    pub async fn get_tags(&self) -> Result<TagList, BackendError> {
        db::article::get_tags(self.conn.as_ref().unwrap()).await 
    }
    
}


/*
impl Server {
    pub fn service(&self, service_kind: Services) -> impl Service {
        use Services::*;

        match service_kind {
            RegisterUser(user_reg) => RegisterUserService { 
                user_reg, 
                secret: self.secret(),
                conn: self.conn.as_ref().unwrap(), 
            },
            _ => unimplemented!(),
        }
    }
}

pub(crate) struct RegisterUserService {
    user_reg: UserReg,
    secret: &'static [u8],
    conn: &'static SqlitePool,
}

#[async_trait]
impl Service for RegisterUserService {
    type ResultType = User;

    async fn execute(&self) -> Result<Self::ResultType, crate::errors::BackendError> {
        match self.user_reg.validate() {
            Ok(_) => (),
            Err(err) => return FromValidatorError::from(err).into(),
        };
    
        let res = match db::user::register_user(self.conn.as_ref().unwrap(), &self.user_reg).await {
            Ok(()) => {
                let mut user: User = self.user_reg.into();
    //            auth::Auth::create_token(&mut user)?;
                user.token = Some(auth::Auth::create_token(&user, &self.secret)?);
                Ok(user)
//                Ok(json!(user.wrap()).into())
            },
            Err(err) => err.into(),
        };
        res          
    }

    fn auth(mut self, _: &str) -> Self {
        self
    }
}
*/