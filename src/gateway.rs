mod dispatch_event;
mod event;

use serde::Deserialize;
use serde_repr::Deserialize_repr;

pub use dispatch_event::{DispatchEvent, DispatchEventKind, Ready};
pub use event::Event;

#[derive(Debug, Deserialize)]
pub struct Gateway {
    pub url: String,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
#[non_exhaustive]
enum OpCode {
    Dispatch = 0,
    Hello = 10,
    HeartbeatAck = 11,
}
