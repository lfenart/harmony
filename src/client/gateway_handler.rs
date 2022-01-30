use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use mio::net::TcpStream;
use mio::Poll;
use serde_json::json;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

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

    pub fn run(mut self) -> Result {
        loop {
            if let Some(heartbeat_interval) = self.heartbeat_interval {
                let now = Instant::now();
                if self.last_heartbeat + heartbeat_interval <= now {
                    if !self.last_heartbeat_ack {
                        self.identify()?;
                        // TODO
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
            Event::Hello(hello_event) => {
                self.heartbeat_interval = Some(hello_event.heartbeat_interval);
                self.identify()?;
            }
            Event::InvalidSession(resumable) => {
                if resumable {
                    self.resume()?;
                } else {
                    self.identify()?;
                }
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
                    let text = x.into_text();
                    println!("event: {:?}", text);
                    events.push(serde_json::from_str::<Event>(&text?)?);
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
