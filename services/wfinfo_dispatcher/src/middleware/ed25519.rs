use crate::models::CheckSignatureError;
use actix_web::{
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform},
    error::PayloadError,
    http::StatusCode,
    web::Bytes,
    Error as ActixError, HttpMessage,
};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use futures::TryStreamExt;
use std::{
    future::{Future, Ready},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};
use tracing::error;

#[derive(Clone, Debug)]
pub struct CheckEd25519Signature {
    public_key: Rc<PublicKey>,
}

impl CheckEd25519Signature {
    pub fn new(public_key: PublicKey) -> Self {
        CheckEd25519Signature {
            public_key: Rc::new(public_key),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CheckEd25519Signature
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = ActixError,
        > + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type InitError = ();
    type Transform = CheckEd25519SignatureMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(CheckEd25519SignatureMiddleware {
            service: Rc::new(service),
            public_key: self.public_key.clone(),
        }))
    }
}

#[derive(Clone, Debug)]
pub struct CheckEd25519SignatureMiddleware<S> {
    service: Rc<S>,
    public_key: Rc<PublicKey>,
}

impl<S> CheckEd25519SignatureMiddleware<S> {
    pub const HEADER_SIGNATURE: &'static str = "x-signature-ed25519";
    pub const HEADER_TIMESTAMP: &'static str = "x-signature-timestamp";
}

impl<S, B> Service<ServiceRequest> for CheckEd25519SignatureMiddleware<S>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = ActixError,
        > + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // Prepare
        let svc = self.service.clone();
        let timestamp = req
            .headers()
            .get(Self::HEADER_TIMESTAMP)
            .ok_or(CheckSignatureError::MissingHeader {
                header_name: Self::HEADER_TIMESTAMP,
                status_code: StatusCode::UNAUTHORIZED,
            })
            .and_then(|timestamp| {
                timestamp
                    .to_str()
                    .map(|timestamp| timestamp.to_owned())
                    .map_err(|_| CheckSignatureError::InvalidTimestamp)
            });
        let signature = req
            .headers()
            .get(Self::HEADER_SIGNATURE)
            .ok_or(CheckSignatureError::MissingHeader {
                header_name: Self::HEADER_SIGNATURE,
                status_code: StatusCode::UNAUTHORIZED,
            })
            .and_then(|signature| {
                signature
                    .to_str()
                    .ok()
                    .and_then(|signature| hex::decode(signature).ok())
                    // .and_then(|signature| signature.try_into().ok())
                    .and_then(|signature| Signature::from_bytes(&signature).ok())
                    .ok_or(CheckSignatureError::InvalidSignature(
                        "<trimmed>".to_string(),
                    ))
            });
        let public_key = self.public_key.clone();

        // Execute
        Box::pin(async move {
            // Get required data
            let mut message = timestamp?;
            let signature = signature?;
            let payload: Vec<_> = req
                .take_payload()
                .try_fold(Vec::new(), |mut acc, cur| async move {
                    acc.extend(cur);
                    Ok(acc)
                })
                .await
                .map_err(|_| {
                    error!("error retrieving request payload");
                    CheckSignatureError::InternalServerError
                })?;
            let body = String::from_utf8(payload.clone()).map_err(|_| {
                CheckSignatureError::BadRequest {
                    message: "invalid request payload".into(),
                }
            })?;

            // Verify signature
            message.push_str(&body);
            public_key
                .verify(message.as_bytes(), &signature)
                .map_err(|_| CheckSignatureError::VerificationFailed)?;

            // Reset payload
            let payload = Bytes::from(payload);
            let stream = futures::stream::once(async move {
                Result::<_, PayloadError>::Ok(payload)
            });
            let new_payload = Payload::Stream {
                payload: Box::pin(stream) as _,
            };
            req.set_payload(new_payload);

            // Execute request
            let resp = svc.call(req).await?;
            Ok(resp)
        })
    }
}
