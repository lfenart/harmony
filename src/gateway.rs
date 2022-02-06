mod dispatch_event;
mod event;
mod intents;

use std::net::TcpStream as StdTcpStream;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use mio::net::TcpStream;
use mio::{Interest, Poll, Token};
use serde::{de, Deserialize};
use serde_json::json;
use serde_repr::Deserialize_repr;
use tungstenite::handshake::HandshakeError;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

use crate::consts::*;
use crate::Result;
pub use dispatch_event::{DispatchEvent, DispatchEventKind, Ready};
pub use event::Event;
pub use intents::Intents;

#[derive(Debug)]
pub struct Gateway {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    poll: Poll,
    intents: Intents,
}

impl Deref for Gateway {
    type Target = WebSocket<MaybeTlsStream<TcpStream>>;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

impl DerefMut for Gateway {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.socket
    }
}

impl Gateway {
    pub fn connect(intents: Intents) -> Result<Self> {
        let url = {
            let url = ureq::get(&api!("/gateway"))
                .call()?
                .into_json::<serde_json::Map<String, serde_json::Value>>()?
                .remove("url")
                .ok_or_else(|| de::Error::missing_field("url"))
                .and_then(String::deserialize)?;
            format!("{}/?v={}&encoding=json", url, API_VERSION)
        };
        let mut stream = {
            let stream = StdTcpStream::connect((GATEWAY_HOSTNAME, GATEWAY_PORT))?;
            stream.set_nonblocking(true)?;
            TcpStream::from_std(stream)
        };
        let poll = Poll::new()?;
        poll.registry()
            .register(&mut stream, Token(0), Interest::READABLE)?;
        let (socket, _) = match tungstenite::client_tls(url, stream) {
            Ok(x) => x,
            Err(HandshakeError::Interrupted(mut mid_handshake)) => loop {
                match mid_handshake.handshake() {
                    Ok(x) => break x,
                    Err(HandshakeError::Interrupted(new_mid_handshake)) => {
                        mid_handshake = new_mid_handshake;
                    }
                    Err(err) => return Err(err.into()),
                }
            },
            Err(err) => return Err(err.into()),
        };
        Ok(Self {
            socket,
            poll,
            intents,
        })
    }

    pub fn close(&mut self) -> Result {
        self.socket.close(None)?;
        let mut events = mio::Events::with_capacity(1);
        loop {
            self.poll.poll(&mut events, None)?;
            loop {
                match self.socket.read_message() {
                    Ok(_) => (),
                    Err(tungstenite::Error::ConnectionClosed) => return Ok(()),
                    Err(tungstenite::Error::Io(err))
                        if err.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        break;
                    }
                    Err(err) => return Err(err.into()),
                }
            }
        }
    }

    pub fn reconnect(&mut self) -> Result {
        println!("reconnect");
        if let Err(err) = self.close() {
            eprintln!("Gateway::close err: {:?}", err);
        }
        let mut gateway = Self::connect(self.intents)?;
        std::mem::swap(self, &mut gateway);
        Ok(())
    }

    pub fn get_events(&mut self) -> Result<Vec<Event>> {
        let mut events = mio::Events::with_capacity(1);
        self.poll
            .poll(&mut events, Some(Duration::from_millis(1)))?;
        if events.is_empty() {
            return Ok(vec![]);
        }
        let mut events = Vec::new();
        loop {
            match self.socket.read_message() {
                Ok(x) => {
                    if let tungstenite::Message::Text(text) = x {
                        println!("event: {:?}", text);
                        if let Ok(event) = serde_json::from_str::<Event>(&text) {
                            events.push(event);
                        }
                    } else {
                        println!("message: {:?}", x);
                    }
                }
                Err(tungstenite::Error::Io(err))
                    if err.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    return Ok(events);
                }
                Err(err) => return Err(err.into()),
            };
        }
    }

    #[inline]
    pub fn heartbeat(&mut self, sequence_number: Option<u64>) -> Result {
        let map = json!({
            "op": 1,
            "d": sequence_number,
        });
        let message = tungstenite::Message::Text(serde_json::to_string(&map).unwrap());
        self.socket.write_message(message)?;
        Ok(())
    }

    #[inline]
    pub fn identify(&mut self, token: &str) -> Result {
        let map = json!({
            "op": 2,
            "d": {
                "token": token,
                "properties": {
                    "$os": std::env::consts::OS,
                    "$browser": "harmony",
                    "$device": "harmony",
                },
                "intents": self.intents,
            }
        });
        let message = tungstenite::Message::Text(serde_json::to_string(&map).unwrap());
        self.socket.write_message(message)?;
        Ok(())
    }

    #[inline]
    pub fn resume(
        &mut self,
        token: &str,
        session_id: &Option<String>,
        sequence_number: Option<u64>,
    ) -> Result {
        let map = json!({
            "op": 6,
            "d": {
                "token": token,
                "session_id": session_id,
                "seq": sequence_number
            }
        });
        let message = tungstenite::Message::Text(serde_json::to_string(&map).unwrap());
        self.socket.write_message(message)?;
        Ok(())
    }
}

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
