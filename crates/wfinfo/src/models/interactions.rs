use serde::{Deserialize, Serialize};
use wfinfo_azure::functions::{HttpInput, HttpOutput, ServiceBusOutput};
use wfinfo_lib::models::{Interaction, InteractionResponse};

#[derive(Clone, Debug, Deserialize)]
pub struct InteractionInputData<R = HttpInput<Interaction>> {
    pub request: R,
}

#[derive(Clone, Debug, Serialize)]
pub struct InteractionOutputData {
    pub response: HttpOutput<InteractionResponse>,
    pub message: ServiceBusOutput,
}
