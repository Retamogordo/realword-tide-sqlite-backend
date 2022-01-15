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
#[serde(rename_all = "camelCase")] 
pub(crate) struct UserUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl std::fmt::Display for UserUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.username.as_ref().map(|val| 
            write!( f, " {}='{}' ", "username", val) ).unwrap_or(Ok(()))?;
        self.email.as_ref().map(|val| write!( f, " {}='{}' ,", "email", val) ).unwrap_or(Ok(()))?;
        self.password.as_ref().map(|val| write!( f, " {}='{}' ,", "password", val) ).unwrap_or(Ok(()))?;
        self.bio.as_ref().map(|val| write!( f, " {}='{}' ,", "bio", val) ).unwrap_or(Ok(()))?;
        self.image.as_ref().map(|val| write!( f, " {}='{}' ,", "image", val) ).unwrap_or(Ok(()))?;
        write!( f, " id=id ")
    }
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

impl User {
    pub fn wrap(self) -> UserWrapped {
        UserWrapped { user: self }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserWrapped {
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(sqlx::FromRow)]
pub(crate) struct Profile {
    username: String,    
    bio: String,    
    image: Option<String>,  
    following: bool,
}

impl Profile {
    pub fn wrap(self) -> ProfileWrapped {
        ProfileWrapped { profile: self }
    }
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
