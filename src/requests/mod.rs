
pub mod user;
pub mod article;

use crate::auth::{Auth, Claims};
use crate::errors::BackendError;

pub(crate) trait AuthenticatedRequest: Sized {
    type FromRequest: IntoAuthenticatedRequest<Self>;
    fn from_request_with_claims(req: Self::FromRequest, claims: Claims) -> Self;
}

pub(crate) trait IntoAuthenticatedRequest<Output: AuthenticatedRequest<FromRequest=Self>> : Sized {
    fn authenticate(self, token: &str, secret: &[u8]) -> Result<Output, BackendError> {
        let claims = Auth::authenticate(token, secret)?;

        Ok(Output::from_request_with_claims(self, claims))
    }
}