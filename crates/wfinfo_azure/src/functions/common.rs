use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FunctionsInput<T> {
    pub data: T,
    // metadata: (),
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FunctionsOutput<T, R = Option<NoReturnValue>> {
    pub outputs: T,
    pub logs: Vec<String>,
    pub return_value: R,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum NoReturnValue {}

impl Serialize for NoReturnValue {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {}
    }
}

impl<'de> Deserialize<'de> for NoReturnValue {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Err(de::Error::custom("value can never be constructed"))
    }
}
