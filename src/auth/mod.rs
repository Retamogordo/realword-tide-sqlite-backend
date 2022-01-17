use tide::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use crate::models::user::{User};
use crate::{errors};

#[derive(Serialize, Deserialize)]
pub(crate) struct Claims {
    pub username: String,
    pub email: String,
    pub exp: i64,
}
pub(crate) struct Auth {
}

impl Auth {
    pub fn create_token(user: &User, secret: &[u8]) -> 
        Result<String, errors::BackendError> {
        let header = Header::new(Algorithm::HS512);
        match chrono::Utc::now() 
            .checked_add_signed(chrono::Duration::minutes(60)) {
                Some(t) => {
                    let exp = t.timestamp();
                    let claims = Claims { 
                        username: user.username.clone(),
                        email: user.email.clone(), 
                        exp
                    };
            
                    let token = encode(
                        &header, 
                        &claims, 
                        &EncodingKey::from_secret(secret))?;
            
                    Ok(token)
                },
                None => Err(errors::BackendError::TokenCreationFailure("time calculation error".to_string()))
            }
    }

    pub fn authenticate(token: &str, secret: &[u8]) -> Result<Claims, errors::BackendError> {
/*        let hdr = req.header(http_types::headers::AUTHORIZATION)
            .ok_or(tide::Error::from_str(tide::StatusCode::Unauthorized, "no authorization header in request"))?
            .get(0)
            .ok_or(tide::Error::from_str(tide::StatusCode::Unauthorized, "no token in request header"))?;
        
        let token = hdr.as_str().trim_start_matches(Self::TOKEN).trim_start();
        */
        let validation = Validation::new(Algorithm::HS512);
//        validation.validate_exp = false;

        let decoded = decode(
                token, 
                &DecodingKey::from_secret(secret), 
                &validation) 
            .map_err(|_|  errors::BackendError::AuthenticationFailure)?;
    //        .map_err(|_| tide::Error::from_str(tide::StatusCode::Unauthorized, "invalid token in request"))?;

        Ok(decoded.claims)
    }
    
    pub fn authorize(token: &str, secret: &[u8], expected_user: &str) 
        -> Result<(), errors::BackendError> {
            let claims = Self::authenticate(token, secret)?;
            if claims.username == expected_user {
                Ok(())
            } else {
                Err(errors::BackendError::Forbidden)
     //           Err(tide::Error::from_str(tide::StatusCode::Forbidden, 
     //               format!("operation not authorized for {}", claims.username)))   
            }
    }
}
