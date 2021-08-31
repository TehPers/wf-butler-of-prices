use actix_web::{http::StatusCode, ResponseError};
use derive_more::{Display, Error};
use wfinfo_lib::http::RequestError;

#[derive(Debug, Display, Error)]
pub enum CommandError {
    #[display(fmt = "error making request")]
    RequestError(RequestError),
}

impl ResponseError for CommandError {
    fn status_code(&self) -> StatusCode {
        match self {
            CommandError::RequestError(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}
