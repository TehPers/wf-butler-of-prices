use async_trait::async_trait;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use truelayer_extensions::Extensions;

#[derive(Clone, Debug, Default)]
pub struct ToErrorMiddleware;

#[async_trait]
impl Middleware for ToErrorMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let res = next.run(req, extensions).await?;
        let res = res.error_for_status()?;
        Ok(res)
    }
}
