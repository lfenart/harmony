#[macro_use]
pub mod macros;

pub mod client;
pub mod consts;
pub mod error;
pub mod gateway;
pub mod http;
pub mod model;
pub mod utils;

pub use error::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;
