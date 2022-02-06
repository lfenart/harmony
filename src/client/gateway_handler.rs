use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use parking_lot::Mutex;
use rand::Rng;

use crate::gateway::{DispatchEvent, Event, Gateway};
use crate::Result;

#[derive(Debug)]
pub struct GatewayHandler {
    token: Arc<str>,
    event_sender: Sender<DispatchEvent>,
    gateway: Arc<Mutex<Gateway>>,
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
        gateway: Arc<Mutex<Gateway>>,
    ) -> Self {
        Self {
            token,
            event_sender,
            gateway,
            sequence_number: None,
            session_id: None,
            last_heartbeat: Instant::now(),
            last_heartbeat_ack: false,
            heartbeat_interval: None,
        }
    }

    pub fn reconnect(&mut self) -> Result {
        self.gateway.lock().reconnect()
    }

    pub fn run(mut self) -> Result {
        self.identify()?;
        loop {
            if let Some(heartbeat_interval) = self.heartbeat_interval {
                let now = Instant::now();
                if self.last_heartbeat + heartbeat_interval <= now {
                    if !self.last_heartbeat_ack {
                        eprintln!("HeartbeatAck not received, reconnecting");
                        self.reconnect()?;
                        self.resume()?;
                    }
                    self.heartbeat()?;
                    self.last_heartbeat = now;
                    self.last_heartbeat_ack = false;
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
                    self.heartbeat()?;
                }
                self.sequence_number = Some(dispatch_event.sequence_number);
                self.event_sender.send(dispatch_event)?;
            }
            Event::Heartbeat => self.heartbeat()?,
            Event::InvalidSession(resumable) => {
                let wait = rand::thread_rng().gen_range(1000..=5000);
                std::thread::sleep(Duration::from_millis(wait));
                self.reconnect()?;
                if resumable {
                    self.resume()?;
                } else {
                    self.identify()?;
                }
            }
            Event::Reconnect => {
                self.reconnect()?;
                self.resume()?;
            }
            Event::Hello(hello_event) => {
                self.heartbeat_interval = Some(hello_event.heartbeat_interval);
            }
            Event::HeartbeatAck => self.last_heartbeat_ack = true,
            Event::Unknown(x) => println!("unknown event: {:?}", x),
        }
        Ok(())
    }

    fn get_events(&mut self) -> Result<Vec<Event>> {
        self.gateway.lock().get_events()
    }

    #[inline]
    fn heartbeat(&mut self) -> Result {
        println!("heartbeat");
        let now = Instant::now();
        self.gateway.lock().heartbeat(self.sequence_number)?;
        self.last_heartbeat = now;
        self.last_heartbeat_ack = false;
        Ok(())
    }

    #[inline]
    fn identify(&mut self) -> Result {
        println!("identify");
        self.gateway.lock().identify(&self.token)?;
        Ok(())
    }

    #[inline]
    fn resume(&mut self) -> Result {
        println!("resume");
        self.gateway
            .lock()
            .resume(&self.token, &self.session_id, self.sequence_number)?;
        Ok(())
    }
}
