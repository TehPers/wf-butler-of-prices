use crate::models::CheckSignatureError;
use actix_web::{
    dev::{
        Payload, PayloadStream, Service, ServiceRequest, ServiceResponse,
        Transform,
    },
    error::PayloadError,
    http::StatusCode,
    web::Bytes,
    Error as ActixError, HttpMessage,
};
use futures::TryStreamExt;
use std::{
    future::{Future, Ready},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct LogBody;

impl<S, B> Transform<S, ServiceRequest> for LogBody
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
    type Transform = LogBodyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(LogBodyMiddleware {
            service: Rc::new(service),
        }))
    }
}

#[derive(Clone, Debug)]
pub struct LogBodyMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LogBodyMiddleware<S>
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

        // Execute
        Box::pin(async move {
            // Get required data
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

            // Reset payload
            let payload = Bytes::from(payload);
            let stream = futures::stream::once(async move {
                Result::<_, PayloadError>::Ok(payload)
            });
            let new_payload =
                Payload::Stream(Box::pin(stream) as PayloadStream);
            req.set_payload(new_payload);

            // Execute request
            let resp = svc.call(req).await?;

            // Log body if response status is 400
            if resp.status() == StatusCode::BAD_REQUEST {
                info!(?body);
            }

            Ok(resp)
        })
    }
}
