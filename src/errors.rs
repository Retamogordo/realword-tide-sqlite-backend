use tide::prelude::*;

const DB_UNIQUE_CONSTRAINT_VIOLATION: &str = "1555";
const SQLITE_CONSTRAINT_UNIQUE: &str = "2067";

#[derive(Debug, Serialize)]
pub enum BackendError {
    UsernameOrEmailExists,
    TokenCreationFailure(String),
    ValidationError(String),
    AuthenticationFailure,
    IncorrectUsernameOrPassword(String),
    Forbidden,
    NoUserFound(String),
    NoArticleFound,
    NoCommentFound(i32),
    NoCommentAdded,
    UnhandledDBError(String, String),
    WebServerConnectionFailure(String),
    UnexpectedError(String),
}


impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
            match self {
                Self::UsernameOrEmailExists => write!( f, "{}", "username or email is already taken"),
                Self::IncorrectUsernameOrPassword(email) => write!( f, "{}", format!("incorrect credentials for {}", email)),
                Self::TokenCreationFailure(message) => write!( f, "{}", format!("JWT not created, reason: {}", message)),
                Self::ValidationError(message) => write!( f, "{}", message),
                Self::AuthenticationFailure => write!( f, "{}", "authentication failure"),
                Self::Forbidden => write!( f, "{}", "operation not authorized"),
                Self::NoUserFound(user_data) => write!( f, "{}", format!("user with {} not found", user_data)),
                Self::NoArticleFound => write!( f, "{}", "article not found"),
                Self::NoCommentFound(id) => write!( f, "{}", format!("comment with id {} not found", id)),
                Self::NoCommentAdded => write!( f, "{}", "no comment added"),
                Self::UnhandledDBError(msg, code) =>  
                    write!( f, "{}", format!("Unhandled db error: {}, code: {}", msg, code)),
                Self::UnexpectedError(msg) => 
                    write!( f, "{}", format!("Unexpected server error occured: {}", msg)),
                Self::WebServerConnectionFailure(msg) =>
                    write!( f, "{}", format!("Web server connection failed: {}", msg)),
            }
    }
}

impl From<validator::ValidationErrors> for BackendError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::ValidationError(err.to_string())
    }
}

impl Into<tide::Result> for BackendError {
    fn into(self) -> tide::Result {
        let message = self.to_string();
        match self {
            Self::ValidationError(_)
            |
            Self::UsernameOrEmailExists 
            |
            Self::IncorrectUsernameOrPassword(_)
            |
            Self::NoUserFound(_)  
            |
            Self::NoArticleFound
            |
            Self::NoCommentAdded
            |
            Self::NoCommentFound(_) => {
                Ok(tide::Response::from(json!({ "errors":{"body": [ message ] }})))    
            }
            Self::UnhandledDBError(_, _)
            |
            Self::TokenCreationFailure(_) => 
                Err(tide::Error::from_str(tide::StatusCode::InternalServerError, 
                    json!({ "errors":{"body": [ message ] }}))),
            Self::AuthenticationFailure => Err(tide::Error::from_str(tide::StatusCode::Unauthorized, self.to_string())),
            Self::Forbidden => Err(tide::Error::from_str(tide::StatusCode::Forbidden, self.to_string())),
            Self::UnexpectedError(_) => Err(tide::Error::from_str(tide::StatusCode::InternalServerError, self.to_string())),
            Self::WebServerConnectionFailure(_) => unreachable!(),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for BackendError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::TokenCreationFailure(err.to_string())
    }
}

impl From<sqlx::Error> for BackendError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(ref db_err) => {
                let code = db_err.code().unwrap().into_owned();
                if DB_UNIQUE_CONSTRAINT_VIOLATION == code 
                    || SQLITE_CONSTRAINT_UNIQUE == code {
                        BackendError::UsernameOrEmailExists
                } else {
                    BackendError::UnhandledDBError(
                        db_err.message().to_string(),
                        code.to_string(),
                    )
                }
            },
            _ => unimplemented!("{}", err),
        }
    }
}

impl From<std::io::Error> for BackendError {
    fn from(err: std::io::Error) -> Self {
        BackendError::WebServerConnectionFailure(err.to_string())
    }
}
