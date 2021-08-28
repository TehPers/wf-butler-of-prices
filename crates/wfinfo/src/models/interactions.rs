use serde::{Deserialize, Serialize};
use wfinfo_azure::functions::{HttpInput, ServiceBusOutput};
use wfinfo_lib::models::Interaction;

#[derive(Clone, Debug, Deserialize)]
pub struct InteractionInputData<R = HttpInput<Interaction>> {
    pub request: R,
}

#[derive(Clone, Debug, Serialize)]
pub struct InteractionOutputData {
    pub message: ServiceBusOutput,
}
