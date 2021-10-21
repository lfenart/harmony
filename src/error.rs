use std::fmt;
use std::io;
use std::sync::mpsc::SendError;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
    Send,
    Tungstenite(tungstenite::Error),
    Ureq(ureq::Error),
    Custom(Box<dyn std::error::Error + Send>),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Json(err) => err.fmt(f),
            Self::Send => "sending on a closed channel".fmt(f),
            Self::Tungstenite(err) => err.fmt(f),
            Self::Ureq(err) => err.fmt(f),
            Self::Custom(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self {
        Self::Send
    }
}

impl From<tungstenite::Error> for Error {
    fn from(error: tungstenite::Error) -> Self {
        Self::Tungstenite(error)
    }
}

impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        Self::Ureq(error)
    }
}
