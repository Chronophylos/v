use rocket::{http::Status, request::FromRequest};

/// Returns true if `key` is a valid API key string.
fn is_valid(key: &str) -> bool {
    key.len() == 16
}

#[derive(Debug, Clone)]
pub struct DeletionToken<'a>(&'a str);

#[derive(Debug, Clone, Copy)]
pub enum DeletionTokenError {
    Missing,
    Invalid,
    BadCount,
}

impl<'a, 'r> FromRequest<'a, 'r> for DeletionToken<'a> {
    type Error = DeletionTokenError;

    fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        use rocket::Outcome::*;

        let keys: Vec<_> = request.headers().get("x-api-key").collect();
        match keys.len() {
            0 => Failure((Status::BadRequest, DeletionTokenError::Missing)),
            1 if is_valid(keys[0]) => Success(DeletionToken(keys[0])),
            1 => Failure((Status::BadRequest, DeletionTokenError::Invalid)),
            _ => Failure((Status::BadRequest, DeletionTokenError::BadCount)),
        }
    }
}
