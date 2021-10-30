mod builder;
mod context;
mod event_handler;
mod gateway_handler;

use std::net::TcpStream as StdTcpStream;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use mio::net::TcpStream;
use mio::{Interest, Poll, Token};
use serde::Deserialize;
use tungstenite::handshake::HandshakeError;

use crate::consts::{API_VERSION, GATEWAY_HOSTNAME, GATEWAY_PORT};
use crate::gateway::{DispatchEvent, Intents, Ready};
use crate::model::Message;
use crate::Result;
pub use builder::ClientBuilder;
pub use context::Context;
use event_handler::EventHandler;
use gateway_handler::GatewayHandler;

pub(crate) type Callback<'a, T> = Box<dyn Fn(Context, T) + 'a>;

pub struct Client<'a> {
    token: String,
    intents: Intents,
    on_ready: Callback<'a, Ready>,
    on_message_create: Callback<'a, Message>,
}

impl<'a> Client<'a> {
    pub fn run(self) -> Result<()> {
        let (gateway_handler, event_handler) = self.connect()?;
        let _ = thread::spawn(move || gateway_handler.run());
        event_handler.run()?;
        Ok(())
    }

    fn connect(self) -> Result<(GatewayHandler, EventHandler<'a>)> {
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
        let token = Arc::<str>::from(self.token);
        let (event_sender, event_receiver) = mpsc::channel::<DispatchEvent>();
        let gateway_handler =
            GatewayHandler::new(token.clone(), event_sender, socket, poll, self.intents);
        let event_handler =
            EventHandler::new(token, event_receiver, self.on_ready, self.on_message_create);
        Ok((gateway_handler, event_handler))
    }
}

#[derive(Deserialize)]
struct Gateway {
    pub url: String,
}
