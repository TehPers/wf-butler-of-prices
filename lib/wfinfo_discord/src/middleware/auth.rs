use crate::{
    models::{ClientCredentialsRequest, Snowflake},
    routes::{AuthenticateClientCredentialsGrant, DiscordRouteInfo},
};
use derive_more::{Display, Error};
use futures::{future::BoxFuture, ready, FutureExt};
use http::StatusCode;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Formatter},
    ops::Deref,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::{Mutex, RwLock};
use tower::{Layer, Service};
use tracing::debug;
use wfinfo_http::{middleware::RestRequestBuilder, RequestError, RestClient};
use zeroize::Zeroizing;

#[derive(Clone, Debug)]
pub struct AuthenticationLayer<C> {
    auth_client: C,
    client_id: Snowflake,
    client_secret: Arc<ClientSecret>,
    access_token: Arc<RwLock<Option<ClientSecret>>>,
}

impl<C> AuthenticationLayer<C> {
    pub fn new(
        auth_client: C,
        client_id: Snowflake,
        client_secret: Arc<ClientSecret>,
    ) -> Self {
        AuthenticationLayer {
            auth_client,
            client_id,
            client_secret,
            access_token: Arc::new(RwLock::new(None)),
        }
    }
}

impl<C, Next> Layer<Next> for AuthenticationLayer<C>
where
    C: Clone,
{
    type Service = AuthenticationService<C, Next>;

    fn layer(&self, next: Next) -> Self::Service {
        AuthenticationService {
            auth_client: self.auth_client.clone(),
            client_id: self.client_id,
            client_secret: self.client_secret.clone(),
            access_token: self.access_token.clone(),
            next: Arc::new(Mutex::new(next)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthenticationService<C, Next> {
    auth_client: C,
    client_id: Snowflake,
    client_secret: Arc<ClientSecret>,
    access_token: Arc<RwLock<Option<ClientSecret>>>,
    next: Arc<Mutex<Next>>,
}

impl<Next, C> Service<RestRequestBuilder> for AuthenticationService<C, Next>
where
    C: RestClient<AuthenticateClientCredentialsGrant>
        + Clone
        + Send
        + Sync
        + 'static,
    Next: Service<RestRequestBuilder, Response = Response> + Send + 'static,
    Next::Error: From<AuthenticationError>,
    Next::Future: Send + 'static,
{
    type Response = Next::Response;
    type Error = Next::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let next = self.next.lock();
        let mut next = ready!(Box::pin(next).poll_unpin(cx));
        next.poll_ready(cx)
    }

    fn call(&mut self, req: RestRequestBuilder) -> Self::Future {
        // Get route info
        let info: &DiscordRouteInfo = match req.get() {
            Some(info) => info,
            None => {
                return Box::pin(async move {
                    Err(AuthenticationError::MissingRouteInfo)?
                })
            }
        };

        // Check if auth is needed
        if !info.needs_auth {
            let next = self.next.clone();
            return Box::pin(async move {
                let mut next = next.lock().await;
                next.call(req).await
            });
        }

        let access_token = self.access_token.clone();
        let auth_client = self.auth_client.clone();
        let client_id = self.client_id;
        let client_secret = self.client_secret.clone();
        let next = self.next.clone();

        Box::pin(async move {
            // Authentication
            debug!("checking for access token");
            let access_token_guard = access_token.read().await;
            let access_token_value =
                if let Some(access_token) = access_token_guard.as_ref() {
                    // Fast path - no need to update access token
                    access_token.clone()
                } else {
                    // Slow path - write lock + verify access token again
                    debug!("checking again for access token");
                    drop(access_token_guard);
                    let mut access_token_guard = access_token.write().await;
                    if let Some(access_token) = access_token_guard.as_ref() {
                        access_token.clone()
                    } else {
                        debug!("fetching credentials");
                        let credentials =
                            AuthenticateClientCredentialsGrant::execute(
                                &auth_client,
                                ClientCredentialsRequest {
                                    grant_type: "client_credentials".to_owned(),
                                    scope: "applications.commands.update"
                                        .to_owned(),
                                },
                                client_id,
                                client_secret.clone(),
                            )
                            .await
                            .map_err(
                                AuthenticationError::ErrorGettingAccessToken,
                            )?;
                        let access_token = access_token_guard
                            .insert(credentials.access_token.into());
                        access_token.clone()
                    }
                };

            // Insert auth header
            let req = req.with_modified_request(|req| {
                req.bearer_auth(&*access_token_value)
            });

            // Execute request
            let mut next = next.lock().await;
            let response = next.call(req).await?;

            // Reset auth token if unauthorized
            if response.status() == StatusCode::UNAUTHORIZED {
                access_token.write().await.take();
            }

            Ok(response)
        })
    }
}

#[derive(Clone, Default)]
pub struct ClientSecret(Zeroizing<String>);

impl Debug for ClientSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ClientSecret").field(&"<secret>").finish()
    }
}

impl From<String> for ClientSecret {
    fn from(value: String) -> Self {
        ClientSecret(Zeroizing::new(value))
    }
}

impl Deref for ClientSecret {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Serialize for ClientSecret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ClientSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Into::into)
    }
}

#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum AuthenticationError {
    #[display(fmt = "error getting access token")]
    ErrorGettingAccessToken(RequestError),
    #[display(fmt = "missing Discord API route info")]
    MissingRouteInfo,
}

impl From<AuthenticationError> for RequestError {
    fn from(error: AuthenticationError) -> Self {
        RequestError::Custom(error.into())
    }
}
