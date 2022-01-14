use tide::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use crate::models::user::{User};

#[derive(Serialize, Deserialize)]
pub(crate) struct Claims {
    pub username: String,
    pub email: String,
    pub exp: i64,
}
pub(crate) struct Auth {
}

impl Auth {
    const TOKEN: &'static str = "Token ";

    pub fn create_token(user: &User, secret: &'static [u8]) -> 
        Result<String, tide::Error> {
        let header = Header::new(Algorithm::HS512);
        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::minutes(60))
            .ok_or(tide::Error::from_str(tide::StatusCode::InternalServerError,"jwt creation error"))?
            .timestamp();
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
    }

    pub fn authenticate(req: &crate::Request) -> Result<(Claims, String), tide::Error> {
        let hdr = req.header(http_types::headers::AUTHORIZATION)
            .ok_or(tide::Error::from_str(tide::StatusCode::Unauthorized, "no authorization header in request"))?
            .get(0)
            .ok_or(tide::Error::from_str(tide::StatusCode::Unauthorized, "no token in request header"))?;
        
        let token = hdr.as_str().trim_start_matches(Self::TOKEN).trim_start();
        let validation = Validation::new(Algorithm::HS512);
//        validation.validate_exp = false;

        let decoded = decode(
                token, 
                &DecodingKey::from_secret(req.state().secret), 
                &validation) 
            .map_err(|_| tide::Error::from_str(tide::StatusCode::Unauthorized, "invalid token in request"))?;

        Ok((decoded.claims, token.to_string()))
    }
    
    pub fn authorize(req: &crate::Request, expected_user: &str) 
        -> Result<(), tide::Error> {
            let (claims, _) = Self::authenticate(req)?;
            if claims.username == expected_user {
                Ok(())
            } else {
                Err(tide::Error::from_str(tide::StatusCode::Forbidden, 
                    format!("operation not authorized for {}", claims.username)))   
            }
    }
}
