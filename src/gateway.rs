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
enum OpCode {
    Dispatch = 0,
    Hello = 10,
    HeartbeatAck = 11,
}
