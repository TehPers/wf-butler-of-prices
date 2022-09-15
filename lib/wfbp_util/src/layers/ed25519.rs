use crate::{errors::ReadBodyError, util::body_to_bytes};
use axum::{
    body::{Body, Bytes, HttpBody},
    http::{header::HeaderName, Request},
};
use derive_more::{Display, From};
use ed25519_dalek::{PublicKey, Verifier};
use futures::{future::BoxFuture, ready, FutureExt};
use std::{
    error::Error,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{BoxError, Layer, Service};

/// Validates the Ed25519 signature of incoming requests.
#[derive(Clone, Debug)]
pub struct CheckEd25519SignatureLayer {
    public_key: Arc<PublicKey>,
}

impl CheckEd25519SignatureLayer {
    /// Creates a new [`CheckEd25519SignatureLayer`] from a public key.
    pub fn new(public_key: Arc<PublicKey>) -> Self {
        CheckEd25519SignatureLayer { public_key }
    }
}

impl<S> Layer<S> for CheckEd25519SignatureLayer {
    type Service = CheckEd25519Signature<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CheckEd25519Signature::new(inner, self.public_key.clone())
    }
}

/// Signature header name
pub const HEADER_SIGNATURE: &'static str = "x-signature-ed25519";

/// Timestamp header name
pub const HEADER_TIMESTAMP: &'static str = "x-signature-timestamp";

/// Validates the Ed25519 signature of a request.
#[derive(Debug)]
pub struct CheckEd25519Signature<S> {
    inner: S,
    public_key: Arc<PublicKey>,
    prepared: Option<S>,
}

impl<S> CheckEd25519Signature<S> {
    /// Creates a new [`CheckEd25519Signature`] from a public key wrapping an
    /// inner service.
    pub fn new(inner: S, public_key: Arc<PublicKey>) -> Self {
        CheckEd25519Signature {
            inner,
            public_key,
            prepared: None,
        }
    }
}

impl<S> Clone for CheckEd25519Signature<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            public_key: self.public_key.clone(),
            prepared: None,
        }
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for CheckEd25519Signature<S>
where
    S: Service<Request<Body>> + Clone + Send + 'static,
    S::Error: Into<BoxError>,
    S::Future: Send + 'static,
    ReqBody: HttpBody<Data = Bytes> + Send + Unpin + 'static,
    ReqBody::Error: Into<anyhow::Error>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::Error>> {
        // Check if already ready
        if self.prepared.is_some() {
            return Poll::Ready(Ok(()));
        }

        // Prepare service
        ready!(self.inner.poll_ready(cx)).map_err(Into::into)?;
        let cloned = self.inner.clone();
        let prepared = std::mem::replace(&mut self.inner, cloned);
        self.prepared = Some(prepared);
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Prepare
        let mut inner = self.prepared.take().expect("service not ready");
        let public_key = self.public_key.clone();

        // Execute
        async move {
            // Get header values
            let timestamp = req
                .headers()
                .get(HEADER_TIMESTAMP)
                .ok_or_else(|| CheckSignatureError::MissingHeader {
                    header_name: HeaderName::from_static(HEADER_TIMESTAMP),
                })?
                .as_bytes()
                .to_owned();
            let signature = req
                .headers()
                .get(HEADER_SIGNATURE)
                .ok_or_else(|| CheckSignatureError::MissingHeader {
                    header_name: HeaderName::from_static(HEADER_SIGNATURE),
                })?
                .to_str()
                .ok()
                .and_then(|signature| hex::decode(signature).ok())
                .and_then(|signature| signature.as_slice().try_into().ok())
                .ok_or(CheckSignatureError::InvalidSignature)?;

            // Get request contents
            let (parts, body) = req.into_parts();
            let body = body_to_bytes(body).await.map_err(|err| {
                CheckSignatureError::RequestBody { inner: err.into() }
            })?;

            // Verify signature
            let mut message = timestamp;
            message.extend_from_slice(&body);
            public_key
                .verify(&message, &signature)
                .map_err(|_| CheckSignatureError::VerificationFailed)?;

            // Call inner service
            let body = Bytes::from(body);
            let req = Request::from_parts(parts, Body::from(body));
            inner.call(req).await.map_err(Into::into)
        }
        .boxed()
    }
}

/// An error that can occur when checking the signature of a request.
#[derive(Debug, Display, From)]
pub enum CheckSignatureError {
    /// There was an error when reading the request body.
    #[display(fmt = "error reading request body")]
    #[from]
    RequestBody { inner: ReadBodyError },

    /// The request is missing a header.
    #[display(fmt = "missing header: {header_name}")]
    MissingHeader { header_name: HeaderName },

    /// The request contains an invalid signature.
    #[display(fmt = "invalid request signature")]
    InvalidSignature,

    /// Verification of the request's signature failed.
    #[display(fmt = "verification failed")]
    VerificationFailed,
}

impl Error for CheckSignatureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CheckSignatureError::RequestBody { inner } => Some(inner),
            CheckSignatureError::MissingHeader { .. } => None,
            CheckSignatureError::InvalidSignature => None,
            CheckSignatureError::VerificationFailed => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use ed25519_dalek::Keypair;
    use tower::{ServiceBuilder, ServiceExt};

    const PRIVATE_KEY: &[u8] = &[
        0xB3, 0x15, 0xA2, 0x2D, 0xD0, 0x79, 0x05, 0x75, 0x04, 0x9E, 0xA3, 0x99,
        0xC5, 0x04, 0x1D, 0x27, 0xC4, 0xDD, 0xE3, 0xFB, 0xA4, 0x2D, 0x67, 0x0F,
        0x51, 0xA4, 0x33, 0xE0, 0x83, 0xE1, 0xEA, 0xAE, 0xB9, 0x31, 0xC7, 0xCD,
        0x87, 0x08, 0x0A, 0xF7, 0xF6, 0x7C, 0x1D, 0x22, 0xEB, 0x87, 0x9A, 0xD3,
        0x3F, 0x66, 0x05, 0xE3, 0x42, 0x0E, 0xD8, 0x74, 0x00, 0xCC, 0xA9, 0x1C,
        0x72, 0x53, 0xE3, 0x90,
    ];
    const BODY: &[u8] = b"Hello, world!";
    const TIMESTAMP: &str = "1234567890";
    const SIGNATURE: &[u8] = &[
        0xED, 0x74, 0x01, 0x6E, 0x4B, 0x5C, 0xA9, 0xF3, 0x2D, 0xAB, 0x96, 0xC3,
        0x00, 0x82, 0x5B, 0xC1, 0x16, 0x67, 0x78, 0xC4, 0x95, 0xF0, 0xAF, 0x3E,
        0x29, 0x15, 0x1A, 0x1A, 0x0E, 0xE6, 0x10, 0xAC, 0xAF, 0x58, 0x8E, 0xDB,
        0x84, 0xE1, 0x4D, 0x64, 0x36, 0x03, 0xAA, 0x7A, 0x74, 0x36, 0x93, 0x36,
        0xD3, 0x3B, 0x04, 0xD3, 0x23, 0x6E, 0x3C, 0x07, 0x40, 0x6F, 0x5B, 0xC5,
        0x63, 0x11, 0x58, 0x07,
    ];

    macro_rules! test_service {
        () => {
            test_service!(|_| {})
        };
        (|$req:pat_param| $assert:expr) => {{
            // Create keypair
            let pair =
                Keypair::from_bytes(PRIVATE_KEY).expect("invalid keypair");
            let public_key = pair.public;

            // Create service
            #[allow(unreachable_code)]
            async fn assert_request(
                $req: Request<Body>,
            ) -> Result<(), CheckSignatureError> {
                $assert;
                Ok(())
            }
            ServiceBuilder::new()
                .layer(CheckEd25519SignatureLayer::new(Arc::new(public_key)))
                .service_fn(assert_request)
        }};
    }

    #[tokio::test]
    async fn service_returns_inner_response_on_valid_signature() {
        // Create service
        let service = test_service!();

        // Create request
        let request = Request::builder()
            .header(HEADER_TIMESTAMP, HeaderValue::from_static(TIMESTAMP))
            .header(
                HEADER_SIGNATURE,
                HeaderValue::from_str(&hex::encode(SIGNATURE))
                    .expect("invalid signature header value"),
            )
            .body(Body::from(Bytes::from(BODY)))
            .expect("invalid request");

        // Execute request
        service.oneshot(request).await.expect("request failed");
    }

    #[tokio::test]
    async fn service_returns_error_on_missing_timestamp_header() {
        // Create service
        let service = test_service!(|_| unreachable!());

        // Create request
        let request = Request::builder()
            .header(
                HEADER_SIGNATURE,
                HeaderValue::from_str(&hex::encode(SIGNATURE))
                    .expect("invalid signature header value"),
            )
            .body(Body::from(Bytes::from(BODY)))
            .expect("invalid request");

        // Execute request
        let response = service.oneshot(request).await;

        // Assert
        let error = response.expect_err("request succeeded");
        let error = error.downcast().expect("invalid error type");
        assert!(matches!(
            *error,
            CheckSignatureError::MissingHeader { header_name }
            if header_name == HEADER_TIMESTAMP
        ));
    }

    #[tokio::test]
    async fn service_returns_error_on_missing_signature_header() {
        // Create service
        let service = test_service!(|_| unreachable!());

        // Create request
        let request = Request::builder()
            .header(HEADER_TIMESTAMP, HeaderValue::from_static(TIMESTAMP))
            .body(Body::from(Bytes::from(BODY)))
            .expect("invalid request");

        // Execute request
        let response = service.oneshot(request).await;

        // Assert
        let error = response.expect_err("request succeeded");
        let error = error.downcast().expect("invalid error type");
        assert!(matches!(
            *error,
            CheckSignatureError::MissingHeader { header_name }
            if header_name == HEADER_SIGNATURE
        ));
    }

    #[tokio::test]
    async fn service_returns_error_on_invalid_signature() {
        // Create service
        let service = test_service!(|_| unreachable!());

        // Create request
        let request = Request::builder()
            .header(HEADER_TIMESTAMP, HeaderValue::from_static(TIMESTAMP))
            .header(HEADER_SIGNATURE, HeaderValue::from_static("invalid"))
            .body(Body::from(Bytes::from(BODY)))
            .expect("invalid request");

        // Execute request
        let response = service.oneshot(request).await;

        // Assert
        let error = response.expect_err("request succeeded");
        let error = error.downcast().expect("invalid error type");
        assert!(matches!(*error, CheckSignatureError::InvalidSignature));
    }

    #[tokio::test]
    async fn service_returns_error_on_failed_validation() {
        // Create service
        let service = test_service!(|_| unreachable!());

        // Create request
        let request = Request::builder()
            .header(HEADER_TIMESTAMP, HeaderValue::from_static(TIMESTAMP))
            .header(
                HEADER_SIGNATURE,
                HeaderValue::from_str(&hex::encode(SIGNATURE))
                    .expect("invalid signature header value"),
            )
            .body(Body::from(Bytes::from(b"invalid".as_slice())))
            .expect("invalid request");

        // Execute request
        let response = service.oneshot(request).await;

        // Assert
        let error = response.expect_err("request succeeded");
        let error = error.downcast().expect("invalid error type");
        assert!(matches!(*error, CheckSignatureError::VerificationFailed));
    }
}
