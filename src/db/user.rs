use sqlx::{Pool};
use sqlx::sqlite::{Sqlite};
use crate::{models, filters, errors};

pub(crate) async fn register_user(
    conn: &Pool<Sqlite>,
//    user: &requests::user::UserReg,
    user: &models::user::User,
) -> Result<models::user::User, crate::errors::BackendError>  {


    sqlx::query(
            "INSERT INTO users (username, email, hashed_password)
            VALUES( ?,	?, ?);\n
            INSERT INTO profiles (username, user_id)
            SELECT username, id FROM users WHERE username=?;
            ")
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.hashed_password)
        .bind(&user.username)
        .execute(conn)    
        .await?;

    get_user(conn, filters::UserFilter::default().username(&user.username))
        .await
}

pub(crate) async fn get_user(
    conn: &Pool<Sqlite>,
    filter: filters::UserFilter<'_>,
) -> Result<models::user::User, crate::errors::BackendError>  {

    let statement = format!("\
        SELECT *, NULL as `token` FROM users \
            LEFT JOIN profiles ON users.username = profiles.username \
            WHERE {};", filter
    );

    let user: models::user::User = sqlx::query_as::<_, models::user::User>(&statement)
        .fetch_optional(conn)   
        .await?
        .ok_or(crate::errors::BackendError::NoUserFound(filter.to_string()))?;
    Ok(user)
}

pub(crate) async fn update_user(
    conn: &Pool<Sqlite>,
    updated_user: &models::user::UserUpdate<'_>,
    filter: filters::UpdateUserFilter<'_>
) -> Result<models::user::User, crate::errors::BackendError>  {
    
    let statement = format!(
        "UPDATE profiles SET {} WHERE user_id=(SELECT id FROM users WHERE {});\n
         UPDATE users SET {} WHERE {} ", 
        updated_user.profile, filter, updated_user, filter);

    let query_res = sqlx::query(&statement)
        .execute(conn)    
        .await?;
    

    if 0 < query_res.rows_affected() {    
        let filter = filters::UserFilter {
            username: updated_user.username.or(filter.username),
            email: updated_user.email.or(filter.email),
//            password: None,
        };
        
        get_user(conn, filter).await
    } else { 
        Err(errors::BackendError::NoUserFound(filter.to_string()))
    }
}

pub(crate) async fn get_profile(
    conn: &Pool<Sqlite>,
    username: &str,
) -> Option<models::user::Profile> {

    let profile = sqlx::query_as::<_, models::user::Profile>(
        &format!(
            "SELECT *, 
                (SELECT COUNT(*)>0 FROM followers WHERE celeb_name = '{}') AS following
            FROM profiles 
            INNER JOIN users ON profiles.username = users.username 
            WHERE profiles.username = '{}';
            ", username, username
        )
    )
    .fetch_optional(conn)   
    .await
    .unwrap_or(None);

    profile
}

pub(crate) async fn follow(conn: &Pool<Sqlite>,
    follower_name: &str,
    celeb_name: &str,
) -> Result<models::user::Profile, crate::errors::BackendError>  {
    
    sqlx::query("INSERT INTO followers (follower_name, celeb_name)
        VALUES( ?,?) ON CONFLICT DO NOTHING;")
        .bind(follower_name)
        .bind(celeb_name)
        .execute(conn)    
        .await?;

    get_profile(conn, celeb_name)
        .await
        .ok_or(crate::errors::BackendError::NoUserFound(celeb_name.to_string()))
}

pub(crate) async fn unfollow(conn: &Pool<Sqlite>,
    follower_name: &str,
    celeb_name: &str,
) -> Result<models::user::Profile, crate::errors::BackendError>  {

    let statement = format!("DELETE FROM followers WHERE follower_name='{}' AND celeb_name='{}';", follower_name, celeb_name);
    sqlx::query(&statement)
        .execute(conn)    
        .await?;

    get_profile(conn, celeb_name)
        .await
        .ok_or(crate::errors::BackendError::NoUserFound(celeb_name.to_string()))

}
