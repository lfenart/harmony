mod builder;
mod context;

use std::net::TcpStream;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

use crate::consts::API_VERSION;
use crate::gateway::{DispatchEvent, DispatchEventKind, Event, Gateway, Ready};
use crate::model::Message;
use crate::Result;
pub use builder::ClientBuilder;
pub use context::Context;

pub struct Client<'a> {
    token: String,
    on_ready: Box<dyn Fn(&Context, &Ready) -> Result + 'a>,
    on_message_create: Box<dyn Fn(&Context, &Message) -> Result + 'a>,
}

impl<'a> Client<'a> {
    pub fn start(mut self) -> Result<()> {
        let gateway = {
            let gateway = ureq::get(&api!("/gateway"))
                .call()?
                .into_json::<Gateway>()?;
            format!("{}/?v={}&encoding=json", gateway.url, API_VERSION)
        };
        let (mut socket, _) = tungstenite::connect(gateway)?;
        let context = Context {
            token: self.token.clone(),
            agent: ureq::agent(),
        };

        // Receive hello
        let event = Self::get_event(&mut socket)?.unwrap();
        let hello_event = event.hello().expect("Expected hello");

        // Send identify
        socket.write_message(Self::identify(&self.token))?;

        // Receive ready
        let event = Self::get_event(&mut socket)?.unwrap();
        let dispatch_event = event.into_dispatch().expect("Expected dispatch");
        let ready = dispatch_event.kind.as_ready().expect("Expected ready");
        let last_sequence_number = dispatch_event.sequence_number;
        let session_id = ready.session_id.clone();
        match socket.get_mut() {
            MaybeTlsStream::Plain(s) => s,
            #[cfg(feature = "native-tls")]
            MaybeTlsStream::NativeTls(s) => s.get_mut(),
            #[cfg(feature = "rustls")]
            MaybeTlsStream::Rustls(s) => s.get_mut(),
            _ => unimplemented!(),
        }
        .set_nonblocking(true)?;
        let (event_sender, event_receiver) = mpsc::channel::<DispatchEvent>();
        event_sender.send(dispatch_event)?;
        let heartbeat_interval = hello_event.heartbeat_interval;
        let token = std::mem::take(&mut self.token);
        let _ = thread::spawn(move || {
            Self::event_loop(
                socket,
                event_sender,
                token,
                session_id,
                heartbeat_interval,
                last_sequence_number,
            )
        });

        loop {
            let event = event_receiver.recv().unwrap();
            let result = match event.kind {
                DispatchEventKind::Ready(ready) => (self.on_ready)(&context, &ready),
                DispatchEventKind::MessageCreate(message) => {
                    (self.on_message_create)(&context, &message)
                }
                DispatchEventKind::Unknown(_) => continue,
            };
            if let Err(err) = result {
                println!("Error: {}", err);
            }
        }
    }

    fn event_loop(
        mut socket: WebSocket<MaybeTlsStream<TcpStream>>,
        event_sender: Sender<DispatchEvent>,
        token: String,
        session_id: String,
        heartbeat_interval: Duration,
        mut last_sequence_number: u64,
    ) -> Result<()> {
        let mut last_heartbeat = Instant::now();
        let mut last_heartbeat_ack = false;
        socket.write_message(Self::heartbeat(last_sequence_number))?;
        loop {
            let now = Instant::now();
            if last_heartbeat + heartbeat_interval <= now {
                if !last_heartbeat_ack {
                    socket.write_message(Self::resume(
                        &token,
                        &session_id,
                        last_sequence_number,
                    ))?;
                }
                socket.write_message(Self::heartbeat(last_sequence_number))?;
                last_heartbeat = now;
                last_heartbeat_ack = false;
            }
            let event = Self::get_event(&mut socket)?;
            if let Some(event) = event {
                match event {
                    Event::Dispatch(dispatch_event) => {
                        last_sequence_number = dispatch_event.sequence_number;
                        event_sender.send(dispatch_event)?;
                    }
                    Event::HeartbeatAck => last_heartbeat_ack = true,
                    Event::Hello(_) => (),
                    Event::Unknown(_) => (),
                }
            }
        }
    }

    fn get_event(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Result<Option<Event>> {
        let msg = match socket.read_message() {
            Ok(x) => x,
            Err(tungstenite::Error::Io(err)) if err.kind() == std::io::ErrorKind::WouldBlock => {
                return Ok(None)
            }
            Err(err) => return Err(err.into()),
        };
        let event = serde_json::from_str::<Event>(&msg.into_text()?)?;
        Ok(Some(event))
    }

    #[inline]
    fn identify(token: &str) -> tungstenite::Message {
        let map = json!({
            "op": 2,
            "d": {
                "token": token,
                "properties": {
                    "$os": std::env::consts::OS,
                    "$browser": "harmony",
                    "$device": "harmony",
                },
                "intents": 1 << 9,
            }
        });
        tungstenite::Message::Text(serde_json::to_string(&map).unwrap())
    }

    #[inline]
    fn heartbeat(last_sequence_number: u64) -> tungstenite::Message {
        let map = json!({
            "op": 1,
            "d": last_sequence_number,
        });
        tungstenite::Message::Text(serde_json::to_string(&map).unwrap())
    }

    #[inline]
    fn resume(token: &str, session_id: &str, last_sequence_number: u64) -> tungstenite::Message {
        let map = json!({
            "op": 6,
            "d": {
                "token": token,
                "session_id": session_id,
                "seq": last_sequence_number
            }
        });
        tungstenite::Message::Text(serde_json::to_string(&map).unwrap())
    }
}
