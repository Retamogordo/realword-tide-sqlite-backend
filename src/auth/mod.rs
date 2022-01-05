use tide::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use crate::errors;

#[derive(Serialize, Deserialize)]
pub(crate) struct Claims {
    pub username: String,
    pub email: String,
}
pub(crate) struct Auth {
}


impl Auth {
    const TOKEN: &'static str = "Token ";
    const JWT_SECRET: &'static [u8] = b"secret";

    pub fn create_token(user: &mut crate::User) -> tide::Result {
        let header = Header::new(Algorithm::HS512);
        let claims = Claims { 
            username: user.username.clone(),
            email: user.email.clone(), 
        };

        if let Ok(token) = encode(
            &header, 
            &claims, 
            &EncodingKey::from_secret(Self::JWT_SECRET)) {
            user.token = Some(token);
            Ok(json!(user).into())
        } else {
            Ok(errors::AuthenticationError::TokenCreationError.into())
        }
    }

    pub fn authorize(req: &crate::Request) -> Result<(Claims, String), errors::AuthenticationError> {
        let hdr = req.header(http_types::headers::AUTHORIZATION)
            .ok_or(errors::AuthenticationError::NoAuthorizationHeaderInRequest)?
            .get(0)
            .ok_or(errors::AuthenticationError::NoTokenInRequestHeader)?;
        
        let token = hdr.as_str().trim_start_matches(Self::TOKEN).trim_start();
        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = false;

        let decoded = decode(
                token, 
                &DecodingKey::from_secret(Self::JWT_SECRET), 
                &validation) 
            .map_err(|_| errors::AuthenticationError::InvalidTokenInRequest)?;

        Ok((decoded.claims, token.to_string()))
    } 
}
