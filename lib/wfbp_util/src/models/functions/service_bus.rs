use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceBusInput {
    // TODO
}

#[derive(Clone, Debug, Serialize, Deserialize, From)]
pub enum ServiceBusOutput {
    Single(String),
    Many(Vec<String>),
}

impl Default for ServiceBusOutput {
    fn default() -> Self {
        ServiceBusOutput::Many(Vec::new())
    }
}
