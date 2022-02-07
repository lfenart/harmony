use std::time::Duration;

use serde::de;
use serde::{Deserialize, Deserializer};

use super::{DispatchEvent, OpCode};

#[derive(Debug)]
pub enum Event {
    Dispatch(DispatchEvent),
    Heartbeat,
    Reconnect,
    InvalidSession(bool),
    Hello(HelloEvent),
    HeartbeatAck,
    Unknown(serde_json::Value),
}

impl Event {
    pub const fn hello(&self) -> Option<&HelloEvent> {
        match self {
            Self::Hello(event) => Some(event),
            _ => None,
        }
    }

    pub const fn dispatch(&self) -> Option<&DispatchEvent> {
        match self {
            Self::Dispatch(event) => Some(event),
            _ => None,
        }
    }

    pub fn into_dispatch(self) -> Option<DispatchEvent> {
        match self {
            Self::Dispatch(event) => Some(event),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut map = serde_json::value::Map::deserialize(deserializer)?;
        let op = map
            .remove("op")
            .ok_or_else(|| de::Error::missing_field("op"))?;
        let op_code = OpCode::deserialize(op);
        (match op_code {
            Ok(OpCode::Dispatch) => {
                DispatchEvent::deserialize(serde_json::Value::from(map)).map(Into::into)
            }
            Ok(OpCode::Heartbeat) => Ok(Self::Heartbeat),
            Ok(OpCode::Reconnect) => Ok(Self::Reconnect),
            Ok(OpCode::InvalidSession) => Ok(Self::InvalidSession(
                map.remove("d")
                    .ok_or_else(|| de::Error::missing_field("d"))
                    .and_then(bool::deserialize)
                    .map_err(de::Error::custom)?,
            )),
            Ok(OpCode::Hello) => {
                let d = map
                    .remove("d")
                    .ok_or_else(|| de::Error::missing_field("d"))?;
                HelloEvent::deserialize(d).map(Into::into)
            }
            Ok(OpCode::HeartbeatAck) => Ok(Self::HeartbeatAck),
            Ok(_) => Ok(Self::Unknown(
                map.remove("d")
                    .ok_or_else(|| de::Error::missing_field("d"))?,
            )),
            Err(err) => Err(err),
        })
        .map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone)]
pub struct HelloEvent {
    pub heartbeat_interval: Duration,
}

impl<'de> Deserialize<'de> for HelloEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut map = serde_json::value::Map::deserialize(deserializer)?;
        let heartbeat_interval = map
            .remove("heartbeat_interval")
            .ok_or_else(|| de::Error::missing_field("heartbeat_interval"))
            .and_then(u64::deserialize)
            .map(Duration::from_millis)
            .map_err(de::Error::custom)?;
        Ok(Self { heartbeat_interval })
    }
}

impl From<HelloEvent> for Event {
    fn from(event: HelloEvent) -> Self {
        Self::Hello(event)
    }
}

impl From<DispatchEvent> for Event {
    fn from(event: DispatchEvent) -> Self {
        Self::Dispatch(event)
    }
}
