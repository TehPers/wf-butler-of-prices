use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateLimit {
    /// A message saying you are being rate limited.
    pub message: String,
    /// The number of seconds to wait before submitting another request.
    pub retry_after: f32,
    /// A value indicating if you are being globally rate limited or not
    pub global: bool,
}
