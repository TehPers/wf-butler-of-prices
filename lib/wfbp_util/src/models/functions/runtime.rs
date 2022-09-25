use serde::{Deserialize, Serialize};

/// An input from the Azure Functions custom handler runtime.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FunctionInput<D, M = ()> {
    /// The data being received from the triggers and bindings.
    pub data: D,
    
    /// Metadata about the triggers.
    pub metadata: M,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FunctionsOutput<T, R = ()> {
    pub outputs: T,
    pub logs: Vec<String>,
    pub return_value: R,
}
