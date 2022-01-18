use tide::prelude::*;
use validator::{Validate};

#[derive(Debug, Deserialize, Validate)]
pub struct UserReg {
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UserRegWrapped {
    pub user: UserReg,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")] 
pub struct UserUpdateRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl Default for UserUpdateRequest {
    fn default() -> Self {
        Self { 
            username: None,
            email: None,
            password: None,
            bio: None,
            image: None,
        }
    }
}


#[derive(Debug, Deserialize)]
pub(crate) struct UserUpdateWrapped {
//    user: String,
    pub user: UserUpdateRequest,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginRequestWrapped {
//    user: String,
    pub user: LoginRequest,
}
