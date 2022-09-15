use serde::Deserialize;

/// An input from the Azure Functions custom handler runtime.
#[derive(Clone, Debug, Deserialize)]
pub struct FunctionInput<D, M = ()> {
    /// The data being received from the triggers and bindings.
    pub data: D,
    
    /// Metadata about the triggers.
    pub metadata: M,
}
