use serde::de;
use serde::{Deserialize, Deserializer};

use crate::model::{Message, User};

#[derive(Debug)]
pub struct DispatchEvent {
    pub sequence_number: u64,
    pub kind: DispatchEventKind,
}

impl<'de> Deserialize<'de> for DispatchEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut map = serde_json::value::Map::deserialize(deserializer)?;
        let sequence_number = map
            .remove("s")
            .ok_or_else(|| de::Error::missing_field("s"))
            .and_then(u64::deserialize)
            .map_err(de::Error::custom)?;
        let event_name = map
            .remove("t")
            .ok_or_else(|| de::Error::missing_field("t"))
            .and_then(String::deserialize)
            .map_err(de::Error::custom)?;
        let d = map
            .remove("d")
            .ok_or_else(|| de::Error::missing_field("d"))?;
        let kind = match event_name.as_str() {
            "READY" => Ready::deserialize(d).map(Into::into),
            "MESSAGE_CREATE" => Message::deserialize(d).map(Into::into),
            _ => Ok(DispatchEventKind::Unknown(d)),
        }
        .map_err(de::Error::custom)?;
        Ok(Self {
            sequence_number,
            kind,
        })
    }
}

#[derive(Debug)]
pub enum DispatchEventKind {
    Ready(Ready),
    MessageCreate(Box<Message>),
    Unknown(serde_json::Value),
}

impl DispatchEventKind {
    pub const fn as_ready(&self) -> Option<&Ready> {
        match self {
            Self::Ready(event) => Some(event),
            _ => None,
        }
    }

    pub fn into_ready(self) -> Option<Ready> {
        match self {
            Self::Ready(event) => Some(event),
            _ => None,
        }
    }

    pub const fn as_message(&self) -> Option<&Message> {
        match self {
            Self::MessageCreate(message) => Some(message),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ready {
    #[serde(rename = "v")]
    pub version: u64,
    pub user: User,
    // pub guilds: Vec<UnavailableGuild>,
    pub session_id: String,
    pub shard: Option<[u64; 2]>,
    // pub application: Application,
}

impl From<Ready> for DispatchEventKind {
    fn from(event: Ready) -> Self {
        Self::Ready(event)
    }
}

impl From<Message> for DispatchEventKind {
    fn from(message: Message) -> Self {
        Self::MessageCreate(Box::new(message))
    }
}

impl From<Box<Message>> for DispatchEventKind {
    fn from(message: Box<Message>) -> Self {
        Self::MessageCreate(message)
    }
}
