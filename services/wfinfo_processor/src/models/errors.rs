use actix_web::{http::StatusCode, ResponseError};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum CommandError {
    #[display(fmt = "{}", _0)]
    ParseError(serde_json::Error),
}

impl ResponseError for CommandError {
    fn status_code(&self) -> StatusCode {
        match self {
            CommandError::ParseError(_) => StatusCode::BAD_REQUEST,
        }
    }
}
