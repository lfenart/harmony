use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize)]
pub struct Activity {
    #[serde(rename = "type")]
    pub kind: ActivityType,
    pub name: String,
}

impl Activity {
    pub fn playing<T: ToString>(name: T) -> Self {
        Self {
            kind: ActivityType::Playing,
            name: name.to_string(),
        }
    }

    pub fn streaming<T: ToString>(name: T) -> Self {
        Self {
            kind: ActivityType::Streaming,
            name: name.to_string(),
        }
    }

    pub fn listening<T: ToString>(name: T) -> Self {
        Self {
            kind: ActivityType::Listening,
            name: name.to_string(),
        }
    }

    pub fn watching<T: ToString>(name: T) -> Self {
        Self {
            kind: ActivityType::Watching,
            name: name.to_string(),
        }
    }

    pub fn custom<T: ToString>(name: T) -> Self {
        Self {
            kind: ActivityType::Custom,
            name: name.to_string(),
        }
    }

    pub fn competing<T: ToString>(name: T) -> Self {
        Self {
            kind: ActivityType::Competing,
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ActivityType {
    Playing = 0,
    Streaming = 1,
    Listening = 2,
    Watching = 3,
    Custom = 4,
    Competing = 5,
}
