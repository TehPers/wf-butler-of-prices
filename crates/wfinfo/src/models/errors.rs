use actix_web::{http::StatusCode, ResponseError};
use derive_more::Display;
use std::borrow::Cow;

#[derive(Clone, Debug, Display)]
pub enum InteractionError {
    #[display(fmt = "{}", message)]
    BadRequest { message: Cow<'static, str> },
    #[display(fmt = "interaction not yet supported")]
    NotImplemented,
    #[display(fmt = "internal server error")]
    InternalServerError,
    #[display(fmt = "missing header: {}", header_name)]
    MissingHeader {
        header_name: &'static str,
        status_code: StatusCode,
    },
    #[display(fmt = "invalid request timestamp")]
    InvalidTimestamp,
    #[display(fmt = "invalid request signature")]
    InvalidSignature,
    #[display(fmt = "unauthorized application ID")]
    UnauthorizedApplication,
}

impl ResponseError for InteractionError {
    fn status_code(&self) -> StatusCode {
        match self {
            InteractionError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            InteractionError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            InteractionError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            InteractionError::MissingHeader { status_code, .. } => *status_code,
            InteractionError::InvalidTimestamp => StatusCode::UNAUTHORIZED,
            InteractionError::InvalidSignature => StatusCode::UNAUTHORIZED,
            InteractionError::UnauthorizedApplication => StatusCode::FORBIDDEN,
        }
    }
}
