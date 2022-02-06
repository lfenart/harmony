mod builder;
mod context;
mod event_handler;
mod gateway_handler;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::gateway::{Gateway, Intents, Ready};
use crate::model::Message;
use crate::Result;
pub use builder::ClientBuilder;
pub use context::Context;
use event_handler::EventHandler;
use gateway_handler::GatewayHandler;
use parking_lot::Mutex;

pub(crate) type Callback<'a, T> = Arc<Mutex<dyn FnMut(Context, T) + 'a>>;

pub struct Client<'a> {
    token: String,
    intents: Intents,
    on_ready: Callback<'a, Ready>,
    on_message_create: Callback<'a, Message>,
}

impl<'a> Client<'a> {
    pub fn run(self) -> Result<()> {
        loop {
            match self.connect() {
                Ok((gateway_handler, event_handler)) => {
                    crossbeam_utils::thread::scope(move |s| {
                        s.spawn(|_| {
                            if let Err(err) = gateway_handler.run() {
                                eprintln!("GatewayHandler::run err: {:?}", err);
                            }
                        });
                        if let Err(err) = event_handler.run() {
                            eprintln!("EventHandler::run err: {:?}", err);
                        }
                    })
                    .ok();
                }
                Err(err) => {
                    eprintln!("Client::connect err: {:?}", err);
                    thread::sleep(Duration::from_secs(5));
                }
            }
        }
    }

    fn connect(&self) -> Result<(GatewayHandler, EventHandler<'a>)> {
        let gateway = Arc::new(Mutex::new(Gateway::connect(self.intents)?));
        let token = Arc::<str>::from(self.token.clone());
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();
        let gateway_handler = GatewayHandler::new(token.clone(), event_sender, gateway.clone());
        let event_handler = EventHandler::new(
            token,
            event_receiver,
            gateway,
            self.on_ready.clone(),
            self.on_message_create.clone(),
        );
        Ok((gateway_handler, event_handler))
    }
}
