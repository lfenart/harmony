use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::sync::mpsc::{RecvError, SendError};

use mio::net::TcpStream;
use tungstenite::handshake::client::ClientHandshake;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::HandshakeError;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
    Recv,
    Send,
    Tungstenite(tungstenite::Error),
    TungsteniteHandshake(Box<HandshakeError<ClientHandshake<MaybeTlsStream<TcpStream>>>>),
    Ureq(ureq::Error),
    Custom(Box<dyn StdError + Send>),
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Json(err) => err.fmt(f),
            Self::Recv => RecvError.fmt(f),
            Self::Send => "sending on a closed channel".fmt(f),
            Self::Tungstenite(err) => err.fmt(f),
            Self::TungsteniteHandshake(err) => err.fmt(f),
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

impl From<RecvError> for Error {
    fn from(_: RecvError) -> Self {
        Self::Recv
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

impl From<HandshakeError<ClientHandshake<MaybeTlsStream<TcpStream>>>> for Error {
    fn from(error: HandshakeError<ClientHandshake<MaybeTlsStream<TcpStream>>>) -> Self {
        Self::TungsteniteHandshake(Box::new(error))
    }
}

impl From<Box<HandshakeError<ClientHandshake<MaybeTlsStream<TcpStream>>>>> for Error {
    fn from(error: Box<HandshakeError<ClientHandshake<MaybeTlsStream<TcpStream>>>>) -> Self {
        Self::TungsteniteHandshake(error)
    }
}

impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        Self::Ureq(error)
    }
}
