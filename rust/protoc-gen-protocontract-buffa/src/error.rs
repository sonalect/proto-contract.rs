//! The failure of a plugin run.

use std::fmt;

/// Why the plugin refused a request. The message names the proto element
/// (file, service, method) or the plugin parameter it is about, and how to
/// fix it; it reaches protoc as `CodeGeneratorResponse.error`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    message: String,
}

impl Error {
    pub(crate) fn new(message: impl Into<String>) -> Error {
        Error {
            message: message.into(),
        }
    }

    /// The message protoc shows to the user.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}
