use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use derive_more::{Display, From};
use std::{borrow::Cow, collections::HashMap};
use wfinfo_azure::functions::{FunctionsOutput, RawHttpOutput};

use super::InteractionOutputData;

#[derive(Clone, Debug, Display)]
pub enum CheckSignatureError {
    #[display(fmt = "{}", message)]
    BadRequest { message: Cow<'static, str> },
    #[display(fmt = "internal server error")]
    InternalServerError,
    #[display(fmt = "missing header: {}", header_name)]
    MissingHeader {
        header_name: &'static str,
        status_code: StatusCode,
    },
    #[display(fmt = "invalid request timestamp")]
    InvalidTimestamp,
    #[display(fmt = "invalid request signature: '{}'", _0)]
    InvalidSignature(String),
    #[display(fmt = "verification failed")]
    VerificationFailed,
    #[display(fmt = "request expired")]
    RequestExpired,
}

impl ResponseError for CheckSignatureError {
    fn status_code(&self) -> StatusCode {
        match self {
            CheckSignatureError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            CheckSignatureError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            CheckSignatureError::MissingHeader { status_code, .. } => {
                *status_code
            }
            CheckSignatureError::InvalidTimestamp
            | CheckSignatureError::InvalidSignature(..)
            | CheckSignatureError::VerificationFailed
            | CheckSignatureError::RequestExpired => StatusCode::UNAUTHORIZED,
        }
    }
}

#[derive(Debug, Display, From)]
pub enum InteractionError {
    #[display(fmt = "{}", message)]
    BadRequest { message: Cow<'static, str> },
    #[display(fmt = "interaction not yet supported")]
    NotImplemented,
    #[display(fmt = "internal server error")]
    InternalServerError,
    #[display(fmt = "invalid signature")]
    InvalidSignature(CheckSignatureError),
    #[display(fmt = "unauthorized application ID")]
    UnauthorizedApplication,
    #[display(fmt = "invalid request body")]
    InvalidBody(serde_json::Error),
}

impl ResponseError for InteractionError {
    fn status_code(&self) -> StatusCode {
        match self {
            InteractionError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            InteractionError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            InteractionError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            InteractionError::InvalidSignature(inner) => inner.status_code(),
            InteractionError::UnauthorizedApplication => StatusCode::FORBIDDEN,
            InteractionError::InvalidBody(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let body = FunctionsOutput {
            outputs: InteractionOutputData { message: vec![] },
            logs: vec![],
            return_value: Some(RawHttpOutput {
                status: self.status_code().as_u16(),
                headers: HashMap::new(),
                body: self.to_string(),
            }),
        };
        dbg!(format!("{:?}", body));
        HttpResponse::Ok().json(body)
    }
}

#[derive(Debug, Display)]
pub enum AdminCommandError {
    #[display(fmt = "{}", message)]
    BadRequest { message: Cow<'static, str> },
    #[display(fmt = "internal server error")]
    InternalServerError,
    #[display(fmt = "command failed")]
    CommandFailed { cause: anyhow::Error },
}

impl ResponseError for AdminCommandError {
    fn status_code(&self) -> StatusCode {
        match self {
            AdminCommandError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            AdminCommandError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AdminCommandError::CommandFailed { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
