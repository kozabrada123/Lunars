use std::error::Error;

use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    Request, Response,
};
use rocket_okapi::{
    okapi::openapi3::{MediaType, Responses},
    util::{add_content_response, set_content_type},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn example_api_error_code() -> u16 {
    ApiError::username_already_taken().code
}

fn example_api_error_message() -> String {
    ApiError::username_already_taken().message
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug, JsonSchema)]
/// Structure representing an api error.
///
/// Code should be used to create an identifier for each possible error.
/// Message is a human readable message of what went wrong.
///
/// Status is used to set the status code.
pub struct ApiError {
    ///
    /// Will be set to 0 if the error contains no further information.
    pub code: u16,
    /// A user readable message of what went wrong.
    pub message: String,
    #[schemars(skip)]
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

    /// Returns an error for when we tried to add a match where one player played against himself
    pub fn invalid_username(error: &str) -> Self {
        ApiError {
            status: Status::BadRequest,
            code: 4,
            message: error.to_string(),
        }
    }

    /// Returns an error for when we tried to add a match where one player played against himself
    pub fn match_player_a_is_player_b() -> Self {
        ApiError {
            status: Status::BadRequest,
            code: 5,
            message:
                "Player a cannot be player b; a player cannot play a ranked match against themself"
                    .to_string(),
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

impl rocket_okapi::response::OpenApiResponderInner for ApiError {
    fn responses(
        gen: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> rocket_okapi::Result<rocket_okapi::okapi::openapi3::Responses> {
        let mut responses = Responses::default();
        set_content_type(&mut responses, ContentType::JSON)?;

        let possible_responses = vec![
            ApiError::from_status(Status::NotFound),
            ApiError::username_already_taken(),
        ];

        let schema = gen.json_schema::<ApiError>();

        for error in possible_responses {
            add_content_response(
                &mut responses,
                error.status.code,
                ContentType::JSON,
                MediaType {
                    schema: Some(schema.clone()),
                    example: Some(serde_json::Value::String(format!(
                        r#"{{ "code": {}, "message": {:?} }}"#,
                        error.code, error.message
                    ))),
                    ..Default::default()
                },
            )?;
        }

        Ok(responses)
    }
}
