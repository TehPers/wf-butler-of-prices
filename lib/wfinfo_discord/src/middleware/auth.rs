use crate::{
    models::{ClientCredentialsRequest, Snowflake},
    routes::{AuthenticateClientCredentialsGrant, DiscordRouteInfo},
};
use async_trait::async_trait;
use derive_more::{Display, Error};
use http::{
    header::{InvalidHeaderValue, AUTHORIZATION},
    HeaderValue, StatusCode,
};
use reqwest_middleware::{Middleware, Next};
use std::{borrow::Cow, sync::Arc};
use tokio::sync::RwLock;
use tracing::debug;
use truelayer_extensions::Extensions;
use wfinfo_lib::{
    http::{
        middleware::{BackoffMiddleware, JitterMiddleware, RetryMiddleware},
        RequestError, StandardRestClient,
    },
    reqwest::{Client, Request, Response},
};

pub struct AuthenticationMiddleware {
    auth_client: StandardRestClient,
    client_id: Snowflake,
    client_secret: Arc<String>,
    access_token: Arc<RwLock<Option<String>>>,
}

impl AuthenticationMiddleware {
    pub fn new(
        raw_client: Client,
        base_url: impl Into<Cow<'static, str>>,
        client_id: Snowflake,
        client_secret: Arc<String>,
    ) -> Self {
        let middleware: [Arc<dyn Middleware>; 3] = [
            Arc::new(RetryMiddleware::default()),
            Arc::new(BackoffMiddleware::default()),
            Arc::new(JitterMiddleware::default()),
        ];

        AuthenticationMiddleware {
            auth_client: StandardRestClient::new_with_middleware(
                raw_client, base_url, middleware,
            ),
            client_id,
            client_secret,
            access_token: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl Middleware for AuthenticationMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        // Get route info
        let info: &DiscordRouteInfo =
            extensions.get().ok_or(MiddlewareError::MissingRouteInfo)?;

        // Check if auth is needed
        if !info.needs_auth {
            return next.run(req, extensions).await;
        }

        // Authentication
        debug!("checking for access token");
        let access_token_guard = self.access_token.read().await;
        let auth_header = if let Some(access_token) =
            access_token_guard.as_ref()
        {
            // Fast path - no need to update access token
            HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(MiddlewareError::InvalidAccessToken)?
        } else {
            // Slow path - write lock + verify access token again
            debug!("checking again for access token");
            drop(access_token_guard);
            let mut access_token_guard = self.access_token.write().await;
            if let Some(access_token) = access_token_guard.as_ref() {
                HeaderValue::from_str(&format!("Bearer {}", access_token))
                    .map_err(MiddlewareError::InvalidAccessToken)?
            } else {
                debug!("fetching credentials");
                let credentials = AuthenticateClientCredentialsGrant::execute(
                    &self.auth_client,
                    ClientCredentialsRequest {
                        grant_type: "client_credentials".to_owned(),
                        scope: "applications.commands.update".to_owned(),
                    },
                    self.client_id,
                    self.client_secret.clone(),
                )
                .await
                .map_err(MiddlewareError::ErrorGettingAccessToken)?;
                let access_token =
                    access_token_guard.insert(credentials.access_token);
                HeaderValue::from_str(&format!("Bearer {}", access_token))
                    .map_err(MiddlewareError::InvalidAccessToken)?
            }
        };

        // Insert auth header
        req.headers_mut().insert(AUTHORIZATION, auth_header);

        // Execute request
        let response = next.run(req, extensions).await?;

        // Reset auth token if unauthorized
        if response.status() == StatusCode::UNAUTHORIZED {
            self.access_token.write().await.take();
        }

        Ok(response)
    }
}

#[derive(Debug, Display, Error)]
#[non_exhaustive]
enum MiddlewareError {
    #[display(fmt = "missing route info for request")]
    MissingRouteInfo,
    #[display(fmt = "invalid access token")]
    InvalidAccessToken(InvalidHeaderValue),
    #[display(fmt = "error getting access token")]
    ErrorGettingAccessToken(RequestError),
}

impl From<MiddlewareError> for reqwest_middleware::Error {
    fn from(error: MiddlewareError) -> Self {
        reqwest_middleware::Error::middleware(error)
    }
}
