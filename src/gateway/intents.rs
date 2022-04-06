use std::ops::BitOr;

use serde::{Serialize, Serializer};

#[derive(Default, Debug, Clone, Copy)]
pub struct Intents(pub u64);

impl Intents {
    pub const GUILDS: Self = Self(1 << 0);
    pub const GUILD_MEMBERS: Self = Self(1 << 1);
    pub const GUILD_BANS: Self = Self(1 << 2);
    pub const GUILD_EMOJIS: Self = Self(1 << 3);
    pub const GUILD_INTEGRATIONS: Self = Self(1 << 4);
    pub const GUILD_WEBHOOKS: Self = Self(1 << 5);
    pub const GUILD_INVITES: Self = Self(1 << 6);
    pub const GUILD_VOICE_STATES: Self = Self(1 << 7);
    pub const GUILD_PRESENCES: Self = Self(1 << 8);
    pub const GUILD_MESSAGES: Self = Self(1 << 9);
    pub const GUILD_MESSAGE_REACTIONS: Self = Self(1 << 10);
    pub const GUILD_MESSAGE_TYPING: Self = Self(1 << 11);
    pub const DIRECT_MESSAGES: Self = Self(1 << 12);
    pub const DIRECT_MESSAGE_REACTIONS: Self = Self(1 << 13);
    pub const DIRECT_MESSAGE_TYPING: Self = Self(1 << 14);
}

impl BitOr<Self> for Intents {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self(self.0 | other.0)
    }
}

impl From<u64> for Intents {
    fn from(x: u64) -> Self {
        Self(x)
    }
}

impl From<Intents> for u64 {
    fn from(intents: Intents) -> Self {
        intents.0
    }
}

impl Serialize for Intents {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(self.0)
    }
}
