use sqlx::{SqliteConnection, Pool};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, Sqlite};
use crate::models::{user, article};

pub(crate) async fn register_user(conn: &Pool<Sqlite>,
    user: &user::UserReg,
) -> Result<(), crate::errors::RegistrationError>  {
    sqlx::query(
            "INSERT INTO users (username, email, password)
            VALUES( ?,	?, ?);\n
            INSERT INTO profiles (username)
            VALUES( ?);
            ")
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.username)
        .execute(conn)    
        .await?;
    Ok(())
}

pub(crate) async fn get_user_by_email(conn: &Pool<Sqlite>,
    email: &str,
) -> Result<user::User, crate::errors::RegistrationError>  {

    let user: user::User = sqlx::query_as::<_, user::User>(
        "SELECT *, NULL as `token` FROM users LEFT JOIN profiles ON users.username = profiles.username WHERE email = ?;")
//            "SELECT *, NULL as `token` FROM users LEFT JOIN profiles ON users.username = profiles.username WHERE users.username = ?;")
        .bind(email)
        .fetch_optional(conn)   
        .await?
        .ok_or(crate::errors::RegistrationError::NoUserFound(email.to_string()))?;
    Ok(user)
}

pub(crate) async fn get_user_by_username(conn: &Pool<Sqlite>,
    username: &str,
) -> Result<user::User, crate::errors::RegistrationError>  {

    let user: user::User = sqlx::query_as::<_, user::User>(
        "SELECT *, NULL as `token` FROM users LEFT JOIN profiles ON users.username = profiles.username WHERE users.username = ?;")
//            "SELECT *, NULL as `token` FROM users LEFT JOIN profiles ON users.username = profiles.username WHERE users.username = ?;")
        .bind(username)
        .fetch_optional(conn)   
        .await?
        .ok_or(crate::errors::RegistrationError::NoUserFound(username.to_string()))?;
    Ok(user)
}

pub(crate) async fn update_user(conn: &Pool<Sqlite>,
    username: &str,
    user: &user::UserUpdate,
) -> Result<user::User, crate::errors::RegistrationError>  {
    // use "dummy" set username=username for case there is nothing to update,
    // probably there is better way to perform this "empty update"
    let statement = "UPDATE users SET username=username, "; 
    let mut s = format!("{}", statement);
 //  let mut email_changed = false;

    let new_username = if let Some(new_username) = user.username.as_ref() {
        s = format!("{} username = '{}',", s, new_username);
        new_username
    } else { username };

    if let Some(email) = user.email.as_ref() {
        s = format!("{} email = '{}',", s, email);
 //       email_changed = true;
    }
    s = format!("{} WHERE username = '{}';", s.split_at(s.len()-1).0, username);
    
    // use "dummy" set bio=bio for case there is nothing to update
    // probably there is better way to perform this "empty update"
    s = format!("{} UPDATE profiles SET bio=bio,", s); 
    if let Some(bio) = user.bio.as_ref() {
        s = format!("{} bio = '{}',", s, bio);
    }
    if let Some(image) = user.username.as_ref() {
        s = format!("{} image = '{}',", s, image);
    }
    s = format!("{} WHERE username = '{}';", s.split_at(s.len()-1).0, new_username);
    
    sqlx::query(&s)
        .execute(conn)    
        .await?;

    get_user_by_username(conn, new_username).await
}

pub(crate) async fn get_profile(conn: &Pool<Sqlite>,
    username: &str,
) -> Option<user::Profile> {
//-> Result<user::Profile, crate::errors::RegistrationError>  {

    let profile = sqlx::query_as::<_, user::Profile>(
            &format!("SELECT *, 
                (SELECT COUNT(*)>0 FROM followers 
                    WHERE celeb_name = '{}'
                    ) AS following
            FROM profiles 
            INNER JOIN users ON profiles.username = users.username 
            WHERE profiles.username = '{}';
    ", username, username))
//        .bind(username)
//        .bind(username)
        .fetch_optional(conn)   
        .await
        .unwrap_or(None);

    profile
//        .ok_or(crate::errors::RegistrationError::NoUserFound(username.to_string()))?;
//    Ok(profile)
}

pub(crate) async fn follow(conn: &Pool<Sqlite>,
    follower_name: &str,
    celeb_name: &str,
) -> Result<user::Profile, crate::errors::RegistrationError>  {
    
    sqlx::query("INSERT INTO followers (follower_name, celeb_name)
        VALUES( ?,?) ON CONFLICT DO NOTHING;")
        .bind(follower_name)
        .bind(celeb_name)
        .execute(conn)    
        .await?;

    get_profile(conn, celeb_name)
        .await
        .ok_or(crate::errors::RegistrationError::NoUserFound(celeb_name.to_string()))
}
pub(crate) async fn unfollow(conn: &Pool<Sqlite>,
    follower_name: &str,
    celeb_name: &str,
) -> Result<user::Profile, crate::errors::RegistrationError>  {

    let statement = format!("DELETE FROM followers WHERE follower_name='{}' AND celeb_name='{}';", follower_name, celeb_name);
    sqlx::query(&statement)
//        .bind(follower_name)
//        .bind(celeb_name)
        .execute(conn)    
        .await?;

    get_profile(conn, celeb_name)
        .await
        .ok_or(crate::errors::RegistrationError::NoUserFound(celeb_name.to_string()))

}
