use serde::de;
use serde::{Deserialize, Deserializer};

macro_rules! id_u64 {
    ($($t:ident,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $t(pub u64);

            impl From<u64> for $t {
                fn from(x: u64) -> Self {
                    Self(x)
                }
            }

            impl From<$t> for u64 {
                fn from(x: $t) -> Self {
                    x.0
                }
            }

            impl<'de> Deserialize<'de> for $t {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    Ok(Self(
                        String::deserialize(deserializer)?
                            .parse()
                            .map_err(de::Error::custom)?,
                    ))
                }
            }
        )*
    };
}

id_u64! {
    ChannelId,
    GuildId,
    MessageId,
    RoleId,
    UserId,
    WebhookId,
}

impl ChannelId {
    pub fn mention(self) -> String {
        format!("<#{}>", self.0)
    }
}

impl RoleId {
    pub fn mention(self) -> String {
        format!("<@&{}>", self.0)
    }
}

impl UserId {
    pub fn mention(self) -> String {
        format!("<@{}>", self.0)
    }
}
