
use tide::prelude::*;
use crate::requests;

#[derive(Debug, Serialize, Clone)]
#[derive(sqlx::FromRow)]
pub struct User {
    pub email: String,    
    pub token: Option<String>,    
    pub username: String,    
    pub bio: String,    
    pub image: Option<String>,  
}

impl From<requests::user::UserReg> for User {
    fn from(user_reg: requests::user::UserReg) -> Self {
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
    pub(crate) fn wrap(self) -> UserWrapped {
        UserWrapped { user: self }
    }
}
#[derive(Debug, Serialize)]
pub(crate) struct UserWrapped {
    pub user: User,
}

#[derive(Debug)]
pub(crate) struct UserUpdate<'a> {
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password: Option<&'a str>,
    pub profile: ProfileUpdate<'a>,
}

impl<'a> From<&'a requests::user::UserUpdateRequest> for UserUpdate<'a> {
    fn from(req: &'a requests::user::UserUpdateRequest) -> Self {
        Self {
            username: req.username.as_deref(),
            email: req.email.as_deref(),
            password: req.password.as_deref(),
            profile: ProfileUpdate { 
                bio: req.bio.as_deref(),
                image: req.image.as_deref(),
            }
        }
    }
}

impl std::fmt::Display for UserUpdate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.username.as_ref().map(|val| 
            write!( f, " {}='{}' ", "username", val) ).unwrap_or(Ok(()))?;
        self.email.as_ref().map(|val| write!( f, " {}='{}' ,", "email", val) ).unwrap_or(Ok(()))?;
        self.password.as_ref().map(|val| write!( f, " {}='{}' ,", "password", val) ).unwrap_or(Ok(()))?;
        write!( f, " id=id ")
    }
}

impl Default for UserUpdate<'_> {
    fn default() -> Self {
        Self { 
            username: None,
            email: None,
            password: None,
            profile: ProfileUpdate::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ProfileUpdate<'a> {
    pub bio: Option<&'a str>,
    pub image: Option<&'a str>,
}

impl std::fmt::Display for ProfileUpdate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bio.as_ref().map(|val| write!( f, " {}='{}' ,", "bio", val) ).unwrap_or(Ok(()))?;
        self.image.as_ref().map(|val| write!( f, " {}='{}' ,", "image", val) ).unwrap_or(Ok(()))?;
        write!( f, " user_id=user_id ")
    }
}

impl Default for ProfileUpdate<'_> {
    fn default() -> Self {
        Self { 
            bio: None,
            image: None,
        }
    }
}


#[derive(Debug, Serialize, Clone)]
#[derive(sqlx::FromRow)]
pub struct Profile {
    pub username: String,    
    pub bio: Option<String>,    
    pub image: Option<String>,  
    pub following: bool,
}

impl Profile {
    pub(crate) fn wrap(self) -> ProfileWrapped {
        ProfileWrapped { profile: self }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ProfileWrapped {
    pub profile: Profile,
}
