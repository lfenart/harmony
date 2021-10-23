#[macro_use]
mod macros;

pub mod client;
pub mod consts;
mod error;
pub mod gateway;
pub mod http;
pub mod model;

pub use error::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;
