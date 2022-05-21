use std::fmt::{Display, Formatter, Result};

use actix_web::{HttpResponse, error::ResponseError};

// We use the `anyhow` crate to paper over diferences between various error and
// result types, but we also want to send errors over the wire to the client.
// This means we need to implement the ResponseError trait from Actix on the
// generalized error type from Anyhow. You can't implement a foreign trait on a
// foreign type in Rust. That's like adding unexpected new methods to Array in
// JS. We declare a minimal wrapper type and implement the appropriate conversions
// on this type instead.

// Declaring the type. It's simple enough that Debug can be implemented on it
// automatically.
#[derive(Debug)]
pub(crate) struct CowError(anyhow::Error);

// The Display trait controls what to_string() on this type returns.
// We just proxy to the underlying error's implementation.
impl Display for CowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

// Autoconversion between anyhow::Error and CowError.
impl From<anyhow::Error> for CowError {
    fn from(error: anyhow::Error) -> Self {
        CowError(error)
    }
}

// Finally, defining how the error should be sent over the wire.
// This should be more complicated in a more complete app, with different
// handling for different error types.
impl ResponseError for CowError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(self.0.to_string())
    }
}
