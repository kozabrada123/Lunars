use rocket::{catch, http::Status, Request};

use crate::response::ApiError;

#[catch(default)]
pub fn default_catcher(status: Status, _req: &Request<'_>) -> ApiError {
    ApiError::from_status(status)
}
