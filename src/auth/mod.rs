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

        user.token = Some( encode(
            &header, 
            &claims, 
            &EncodingKey::from_secret(Self::JWT_SECRET))?
        );

        Ok(json!(user).into())
    }

    pub fn authorize(req: &crate::Request) -> Result<(Claims, String), tide::Error> {
        let hdr = req.header(http_types::headers::AUTHORIZATION)
            .ok_or(tide::Error::from_str(tide::StatusCode::Unauthorized, "no authorization header in request"))?
            .get(0)
            .ok_or(tide::Error::from_str(tide::StatusCode::Unauthorized, "no token in request header"))?;
        
        let token = hdr.as_str().trim_start_matches(Self::TOKEN).trim_start();
        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = false;

        let decoded = decode(
                token, 
                &DecodingKey::from_secret(Self::JWT_SECRET), 
                &validation) 
            .map_err(|_| tide::Error::from_str(tide::StatusCode::Unauthorized, "invalid token in request"))?;

        Ok((decoded.claims, token.to_string()))
    } 
}
