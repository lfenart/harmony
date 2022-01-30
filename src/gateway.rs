mod dispatch_event;
mod event;
mod intents;

use serde_repr::Deserialize_repr;

pub use dispatch_event::{DispatchEvent, DispatchEventKind, Ready};
pub use event::Event;
pub use intents::Intents;

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
#[non_exhaustive]
pub enum OpCode {
    Dispatch = 0,
    Heartbeat = 1,
    Reconnect = 7,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
}
