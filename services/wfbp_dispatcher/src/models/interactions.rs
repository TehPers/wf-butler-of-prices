use serde::{Deserialize, Serialize};
use wfbp_azure::functions::{HttpInput, ServiceBusOutput};
use wfbp_discord::models::Interaction;

#[derive(Clone, Debug, Deserialize)]
pub struct InteractionInputData<R = HttpInput<Interaction>> {
    pub request: R,
}

#[derive(Clone, Debug, Serialize)]
pub struct InteractionOutputData {
    pub message: ServiceBusOutput,
}
