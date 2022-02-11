use derive_more::{Display, Error, From};

#[derive(Debug, Display, Error, From)]
#[non_exhaustive]
pub enum RequestError {
    #[display(fmt = "{}", _0)]
    ReqwestError(reqwest::Error),
    #[display(fmt = "{}", _0)]
    Custom(#[error(ignore)] anyhow::Error),
}
