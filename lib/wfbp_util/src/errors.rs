use derive_more::Display;
use std::error::Error;

#[derive(Debug, Display)]
#[display(fmt = "unable to read request body")]
pub struct ReadBodyError(anyhow::Error);

impl ReadBodyError {
    /// Creates a [`ReadBodyError`] from an error.
    pub fn from_error<E>(err: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        Self(err.into())
    }
}

impl Error for ReadBodyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}
