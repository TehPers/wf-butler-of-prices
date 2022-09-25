use serde::{Serialize, Deserialize};
use wfbp_util::models::functions::{HttpInput, ServiceBusOutput};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use derive_more::{Display, From};
use std::{convert::Infallible, error::Error};
use wfbp_util::{
    errors::ReadBodyError,
    layers::CheckSignatureError,
    models::functions::{HttpOutput, FunctionsOutput},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InteractionInput {
    pub request: HttpInput,
}

#[derive(Clone, Debug,  Default, Serialize, Deserialize)]
pub struct InteractionOutput {
    pub message: ServiceBusOutput,
}

#[derive(Debug, Display, From)]
pub enum InteractionError {
    #[display(fmt = "error reading request body")]
    #[from]
    RequestBody { inner: ReadBodyError },
    #[display(fmt = "interaction not yet supported")]
    #[allow(dead_code)] // Keep around in case it's needed later
    NotImplemented,
    #[display(fmt = "internal server error")]
    InternalServerError,
    #[display(fmt = "invalid signature: {_0}")]
    #[from]
    InvalidSignature(CheckSignatureError),
    #[display(fmt = "unauthorized application ID")]
    UnauthorizedApplication,
    #[display(fmt = "{_0}")]
    #[from]
    ParseBody(serde_json::Error),
}

impl From<Infallible> for InteractionError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl Error for InteractionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InteractionError::RequestBody { inner } => Some(inner),
            InteractionError::NotImplemented => None,
            InteractionError::InternalServerError => None,
            InteractionError::InvalidSignature(inner) => Some(inner),
            InteractionError::UnauthorizedApplication => None,
            InteractionError::ParseBody(inner) => Some(inner),
        }
    }
}

impl IntoResponse for InteractionError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            InteractionError::RequestBody { .. } => StatusCode::BAD_REQUEST,
            InteractionError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            InteractionError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            InteractionError::InvalidSignature(inner) => match inner {
                CheckSignatureError::RequestBody { .. } => {
                    StatusCode::BAD_REQUEST
                }
                CheckSignatureError::MissingHeader { .. }
                | CheckSignatureError::InvalidSignature
                | CheckSignatureError::VerificationFailed => {
                    StatusCode::UNAUTHORIZED
                }
            },
            InteractionError::UnauthorizedApplication => StatusCode::FORBIDDEN,
            InteractionError::ParseBody(_) => StatusCode::BAD_REQUEST,
        };
        let body = FunctionsOutput {
            outputs: InteractionOutput::default(),
            logs: vec![],
            return_value: Some(HttpOutput {
                status: status_code,
                body: self.to_string(),
                ..Default::default()
            }),
        };
        (status_code, Json(body)).into_response()
    }
}

