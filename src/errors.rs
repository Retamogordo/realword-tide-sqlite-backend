use tide::prelude::*;
use std::fmt::Display;

const DB_UNIQUE_CONSTRAINT_VIOLATION: &str = "1555";
const SQLITE_CONSTRAINT_UNIQUE: &str = "2067";

pub(crate) struct FromValidatorError(pub validator::ValidationErrors);

impl Into<tide::Result> for FromValidatorError {
    fn into(self) -> tide::Result {
       //let message = self.0.clone().to_string();
//        let err = tide::Error::from_str(tide::StatusCode::UnprocessableEntity, "");
        Ok(tide::Response::from(json!({ "errors":{"body": [ self.0.to_string() ] }})))    
//        let mut response: tide::Response = err.into();
//        response.set_body(json!({ "errors":{"body": [ self.0.to_string() ] }}));
//        Ok(response)
    }
}

impl From<validator::ValidationErrors> for FromValidatorError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self(err)
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum RegistrationError {
//    InvalidEmail,
    UsernameOrEmailExists,
    NoUserFound(String),
    NoArticleFound,
    NoCommentFound,
    UnhandledDBError(String),
}


impl std::fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!( f, "{}", 
            match self {
                Self::UsernameOrEmailExists => "username or email is already taken".to_string(),
                Self::NoUserFound(email) => format!("user with {} not found", email),
                Self::NoArticleFound => "article not found".to_string(),
                Self::NoCommentFound => "comment not found".to_string(),
                Self::UnhandledDBError(msg) =>  
                        format!("Unhandled db error: {}", msg),
            }
        )
    }
}

impl Into<tide::Result> for RegistrationError {
    fn into(self) -> tide::Result {
        let message = self.to_string();
        match self {
//            Self::InvalidEmail => {
//                "email is invalid".to_string()
//            },
            Self::UsernameOrEmailExists 
            |
            Self::NoUserFound(_)  
            |
            Self::NoArticleFound
            |
            Self::NoCommentFound => {
                Ok(tide::Response::from(json!({ "errors":{"body": [ message ] }})))    
            }
            Self::UnhandledDBError(_) => 
//                tide::StatusCode::InternalServerError, 
                Err(tide::Error::from_str(tide::StatusCode::InternalServerError, 
                    json!({ "errors":{"body": [ message ] }})))    
        }
//        tide::Error::from_str(status, json!({ "errors":{"body": [ message ] }}))
    }
}

impl From<sqlx::Error> for RegistrationError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(ref db_err) => {
                let code = db_err.code().unwrap().into_owned();
                if DB_UNIQUE_CONSTRAINT_VIOLATION == code 
                    || SQLITE_CONSTRAINT_UNIQUE == code {
                    RegistrationError::UsernameOrEmailExists
                } else {
                    RegistrationError::UnhandledDBError(db_err.message().to_string())
                }
            },
            _ => unimplemented!("{}", err),
        }
    }
}
/*
#[derive(Debug, Serialize)]
pub(crate) enum AuthenticationError {
    TokenCreationError,
    NoAuthorizationHeaderInRequest,
    NoTokenInRequestHeader,
    InvalidTokenInRequest,
}

impl std::error::Error for AuthenticationError {
}

impl std::fmt::Display for AuthenticationError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::TokenCreationError => {
                "token creation error".to_string()
            },
            Self::NoAuthorizationHeaderInRequest => {
                "no authorization header in request".to_string()
            },
            Self::NoTokenInRequestHeader => {
                "no authentication token in header".to_string()
            },
            Self::InvalidTokenInRequest => {
                "invalid token in request".to_string()
            },
        };
        write!(f, "{}", message)
    }
}

impl Into<tide::Error> for AuthenticationError {
    fn into(self) -> tide::Error {
        Err(tide::Error::from_str(tide::StatusCode::Unauthorized, self.to_string()))
    }
}
*/