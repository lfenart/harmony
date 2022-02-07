use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Online,
    #[serde(rename = "dnd")]
    DoNotDisturb,
    Idle,
    Invisible,
    Offline,
}
