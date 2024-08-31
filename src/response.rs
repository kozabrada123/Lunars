use std::error::Error;

use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    Request, Response,
};

#[derive(Clone, PartialEq, Eq, Debug)]
/// Structure representing an api error.
///
/// Code should be used to create an identifier for each possible error.
/// Message is a human readable message of what went wrong.
///
/// Status is used to set the status code.
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub status: Status,
}

impl ApiError {
    pub fn new(code: u16, message: String, status: Status) -> Self {
        ApiError {
            message,
            code,
            status,
        }
    }

    /// Creates an error with no further information than the status itself.
    pub fn from_status(status: Status) -> Self {
        ApiError {
            status,
            code: 0, // Here the code is set to 0, which means we should look at the http status
            message: status.reason().unwrap_or_default().to_string(),
        }
    }

	 /// Returns an error for when a user sent invalid auth credentials
    pub fn invalid_auth() -> Self {
        ApiError {
            status: Status::Unauthorized,
            code: 1,
            message: "Invalid credentials.".to_string(),
        }
    }

    /// Returns an error for when we try to add a user with an existing username
    pub fn username_already_taken() -> Self {
        ApiError {
            status: Status::BadRequest,
            code: 3,
            message: "A user with that username already exists.".to_string(),
        }
    }
}

impl Error for ApiError {}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{} - {}: {}", self.status.code, self.code, self.message);
        f.write_str(&s)
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let string = format!(
            r#"{{ "code": {}, "message": {:?} }}"#,
            self.code, self.message
        );
        Response::build_from(string.respond_to(req)?)
            .header(ContentType::JSON)
            .status(self.status)
            .ok()
    }
}
