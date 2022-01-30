use std::net::TcpStream as StdTcpStream;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use mio::net::TcpStream;
use mio::{Interest, Poll, Token};
use serde::Deserialize;
use serde_json::json;
use tungstenite::handshake::HandshakeError;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

use crate::consts::*;
use crate::gateway::{DispatchEvent, Event, Intents};
use crate::Result;

pub struct GatewayHandler {
    token: Arc<str>,
    event_sender: Sender<DispatchEvent>,
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    poll: Poll,
    intents: Intents,
    sequence_number: Option<u64>,
    last_heartbeat: Instant,
    heartbeat_interval: Option<Duration>,
    last_heartbeat_ack: bool,
    session_id: Option<String>,
}

impl GatewayHandler {
    pub fn new(
        token: Arc<str>,
        event_sender: Sender<DispatchEvent>,
        socket: WebSocket<MaybeTlsStream<TcpStream>>,
        poll: Poll,
        intents: Intents,
    ) -> Self {
        Self {
            token,
            event_sender,
            socket,
            poll,
            intents,
            sequence_number: None,
            session_id: None,
            last_heartbeat: Instant::now(),
            last_heartbeat_ack: false,
            heartbeat_interval: None,
        }
    }

    fn reconnect(&mut self) -> Result {
        self.socket.close(None)?;
        let gateway = {
            let url = ureq::get(&api!("/gateway"))
                .call()?
                .into_json::<Gateway>()?
                .url;
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
        let (socket, _) = match tungstenite::client_tls(gateway, stream) {
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
        self.poll = poll;
        self.socket = socket;
        Ok(())
    }

    pub fn run(mut self) -> Result {
        loop {
            if let Some(heartbeat_interval) = self.heartbeat_interval {
                let now = Instant::now();
                if self.last_heartbeat + heartbeat_interval <= now {
                    if !self.last_heartbeat_ack {
                        self.reconnect()?;
                    }
                    self.last_heartbeat = now;
                    self.last_heartbeat_ack = false;
                    self.heartbeat()?;
                }
            }
            let events = self.get_events()?;
            for event in events {
                self.handle_event(event)?;
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Result {
        match event {
            Event::Dispatch(dispatch_event) => {
                if let Some(ready) = dispatch_event.kind.as_ready() {
                    self.session_id = Some(ready.session_id.clone());
                    self.last_heartbeat = Instant::now();
                    self.heartbeat()?;
                }
                self.sequence_number = Some(dispatch_event.sequence_number);
                self.event_sender.send(dispatch_event)?;
            }
            Event::Heartbeat => {
                self.last_heartbeat = Instant::now();
                self.heartbeat()?;
            }
            Event::InvalidSession(resumable) => {
                if resumable {
                    self.resume()?;
                } else {
                    self.reconnect()?;
                }
            }
            Event::Reconnect => self.reconnect()?,
            Event::Hello(hello_event) => {
                self.heartbeat_interval = Some(hello_event.heartbeat_interval);
                self.identify()?;
            }
            Event::HeartbeatAck => self.last_heartbeat_ack = true,
            Event::Unknown(_) => (),
        }
        Ok(())
    }

    fn get_events(&mut self) -> Result<Vec<Event>> {
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
                    let text = x.into_text()?;
                    println!("event: {:?}", text);
                    if let Ok(event) = serde_json::from_str::<Event>(&text) {
                        events.push(event);
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
    fn heartbeat(&mut self) -> Result {
        println!("heartbeat");
        let map = json!({
            "op": 1,
            "d": self.sequence_number,
        });
        let message = tungstenite::Message::Text(serde_json::to_string(&map).unwrap());
        self.socket.write_message(message)?;
        Ok(())
    }

    #[inline]
    fn identify(&mut self) -> Result {
        println!("identify");
        let map = json!({
            "op": 2,
            "d": {
                "token": &*self.token,
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
    fn resume(&mut self) -> Result {
        println!("resume");
        let map = json!({
            "op": 6,
            "d": {
                "token": &*self.token,
                "session_id": self.session_id,
                "seq": self.sequence_number
            }
        });
        let message = tungstenite::Message::Text(serde_json::to_string(&map).unwrap());
        self.socket.write_message(message)?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct Gateway {
    url: String,
}
