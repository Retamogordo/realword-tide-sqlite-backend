use tide::prelude::*;

use crate::{models::user};
use crate::endpoints::Request;

#[derive(Debug, Serialize)]
pub struct TestLoginRequestWrapped {
//    pub user: user::LoginRequest,
}
