use tide::prelude::*;
use std::error::Error;

const DB_UNIQUE_CONSTRAINT_VIOLATION: &str = "1555";
const SQLITE_CONSTRAINT_UNIQUE: &str = "2067";

pub(crate) struct FromValidatorError(pub validator::ValidationErrors);

impl Into<tide::Response> for FromValidatorError {
    fn into(self) -> tide::Response {
       //let message = self.0.clone().to_string();
        let err = tide::Error::from_str(tide::StatusCode::UnprocessableEntity, "");
        let mut response: tide::Response = err.into();
        response.set_body(json!({ "errors":{"body": [ self.0.to_string() ] }}));
        response
    }
}

impl From<validator::ValidationErrors> for FromValidatorError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self(err)
    }
}

#[derive(Serialize)]
pub(crate) enum RegistrationError {
//    InvalidEmail,
    UsernameOrEmailExists,
    NoUserFound(String),
    NoArticleFound,
    UnhandledDBError(String),
}

//impl Into<tide::Error> for RegistrationError {
impl Into<tide::Result> for RegistrationError {
    fn into(self) -> tide::Result {
        match self {
//            Self::InvalidEmail => {
//                "email is invalid".to_string()
//            },
            Self::UsernameOrEmailExists => {
                let message = "username or email is already taken".to_string();
                let err = tide::Error::from_str(tide::StatusCode::UnprocessableEntity, message.clone());
                let mut response: tide::Response = err.into();
                response.set_body(json!({ "errors":{"body": [ message ] }}));
                Ok(response)
            },
            Self::NoUserFound(email) => {
                Err(tide::Error::from_str(tide::StatusCode::NotFound, "user not found"))            },
            Self::NoArticleFound => {
                Err(tide::Error::from_str(tide::StatusCode::NotFound, "article not found"))            },
            Self::UnhandledDBError(msg) => {
                Err(tide::Error::from_str(tide::StatusCode::InternalServerError, 
                    format!("Unhandled db error: {}", msg)))            
            },              
        }
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

#[derive(Serialize)]
pub(crate) enum AuthenticationError {
    TokenCreationError,
    NoAuthorizationHeaderInRequest,
    NoTokenInRequestHeader,
    InvalidTokenInRequest,
}

impl Into<tide::Response> for AuthenticationError {
    fn into(self) -> tide::Response {
        let message = match self {
            Self::TokenCreationError => {
                let mut response = tide::Response::from(tide::StatusCode::UnprocessableEntity);
                response.set_body(json!({ "errors":{"body": [ "authentication token not created".to_string() ] }}));
                return response;
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
        let mut response = tide::Response::from(tide::StatusCode::Unauthorized);
        response.set_body(json!({ "errors":{"body": [ message ] }}));
        response
    }
}
