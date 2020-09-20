use std::ops::Deref;

use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};

/// Returns true if `key` is a valid API key string.
fn is_valid(key: &str) -> bool {
    key.len() == 16 && key.is_ascii()
}

#[derive(Debug, Clone)]
pub struct DeletionToken(String);

impl Deref for DeletionToken {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeletionTokenError {
    Missing,
    Invalid,
}

impl FromRequest<'_, '_> for DeletionToken {
    type Error = DeletionTokenError;

    fn from_request(request: &'_ Request<'_>) -> Outcome<Self, Self::Error> {
        use rocket::Outcome::*;
        let value: Option<String> = request
            .get_query_value("deletion_token")
            .and_then(|r| r.ok());
        match value {
            Some(token) if is_valid(&token) => Success(DeletionToken(token)),
            Some(_) => Failure((Status::BadRequest, DeletionTokenError::Invalid)),
            None => Failure((Status::BadRequest, DeletionTokenError::Missing)),
        }
    }
}
