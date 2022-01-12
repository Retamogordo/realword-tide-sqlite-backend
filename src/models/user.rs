use tide::prelude::*;
//use sqlx::prelude::*;
use validator::{Validate};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub(crate) struct UserReg {
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserRegWrapped {
    pub user: UserReg,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserUpdateWrapped {
//    user: String,
    pub user: UserUpdate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(sqlx::FromRow)]
pub(crate) struct User {
    pub email: String,    
    pub token: Option<String>,    
    pub username: String,    
    pub bio: String,    
    pub image: Option<String>,  
}

impl From<UserReg> for User {
    fn from(user_reg: UserReg) -> Self {
        Self {
            username: user_reg.username,
            email: user_reg.email,
            token: None,
            bio: "".to_string(), 
            image: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserWrapped {
    pub user: User,
}
/*
impl UserWrapped {
    fn from_user(user: User) -> Self {
        Self { user }
    }
}
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(sqlx::FromRow)]
pub(crate) struct Profile {
    username: String,    
    bio: String,    
    image: Option<String>,  
    following: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProfileWrapped {
    pub profile: Profile,
}
/*
impl ProfileWrapped {
    fn from_profile(profile: Profile) -> Self {
        Self { profile }
    }
}
*/
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LoginRequestWrapped {
//    user: String,
    pub user: LoginRequest,
}
