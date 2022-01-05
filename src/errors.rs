use tide::prelude::*;

const DB_UNIQUE_CONSTRAINT_VIOLATION: &str = "1555";

#[derive(Serialize)]
pub(crate) enum RegistrationError {
    InvalidEmail,
    UsernameOrEmailExists,
    NoUserFound(String),
    UnhandledDBError(String),
}

//impl Into<tide::Error> for RegistrationError {
impl Into<tide::Response> for RegistrationError {
    fn into(self) -> tide::Response {
        let message = match self {
            Self::InvalidEmail => {
                "email is invalid".to_string()
            },
            Self::UsernameOrEmailExists => {
                "username or email is already taken".to_string()
            },
            Self::NoUserFound(email) => {
                format!("no user found with email {}", email)
            },
            Self::UnhandledDBError(msg) => {
                format!("Unhandled db error: {}", msg)
            }
        };
        let err = tide::Error::from_str(tide::StatusCode::UnprocessableEntity, message.clone());
        let mut response: tide::Response = err.into();
        response.set_body(json!({ "errors":{"body": [ message ] }}));
        response
    }
}

impl From<sqlx::Error> for RegistrationError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(ref db_err) => {
                if DB_UNIQUE_CONSTRAINT_VIOLATION == db_err.code().unwrap().into_owned() {
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
