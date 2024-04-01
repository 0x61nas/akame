pub mod context;
pub mod header;
pub mod macros;
pub mod preload;
pub mod request;
pub mod response;
pub mod router;
pub mod server;

// re-exports
pub use context::Context;
pub use http::Method;
pub use http::StatusCode;
pub use request::Request;
pub use response::Response;
pub use router::Router;
pub use server::Server;

// For macros
#[cfg(feature = "serde_json")]
#[doc(hidden)]
pub use serde_json::json as _json;
#[doc(hidden)]
pub use tokio::fs as _fs;

const CRLF: &str = "\r\n";

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("{0}")]
    IOError(#[from] tokio::io::Error),
    #[error("{0}")]
    InvalidMethod(#[from] http::method::InvalidMethod),
    #[error("Invalid header: `{0}`")]
    InvalidHeader(String),
    #[error("Invalid request")]
    InvalidRequest,
    #[error("Invalid response: `{0}`")]
    InvalidResponse(#[from] http::Error),
}

pub type Result<T> = std::result::Result<T, crate::Error>;
