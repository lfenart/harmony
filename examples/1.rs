use std::collections::HashMap;
use std::env;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use harmony::client::{ClientBuilder, Context};
use harmony::gateway::{Intents, Ready};
use harmony::model::id::{ChannelId, UserId};
use harmony::model::Message;
use parking_lot::Mutex;

struct Lobby {
    size: usize,
    players: HashMap<UserId, DateTime<Utc>>,
}

impl Lobby {
    fn new(size: usize) -> Self {
        Self {
            size,
            players: HashMap::default(),
        }
    }

    fn insert(&mut self, user_id: UserId, date_time: DateTime<Utc>) -> Option<DateTime<Utc>> {
        self.players.insert(user_id, date_time)
    }

    fn remove(&mut self, user_id: &UserId) -> Option<DateTime<Utc>> {
        self.players.remove(user_id)
    }

    fn len(&self) -> usize {
        self.players.len()
    }
}

struct Lobbies(HashMap<ChannelId, Lobby>);

impl Lobbies {
    fn new() -> Self {
        Self(HashMap::default())
    }
}

impl Deref for Lobbies {
    type Target = HashMap<ChannelId, Lobby>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Lobbies {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
struct Error {
    channel_id: ChannelId,
    kind: ErrorKind,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug)]
enum ErrorKind {
    NotALobby(ChannelId),
    AlreadyInQueue(UserId),
    NotInQueue(UserId),
    Harmony(Box<harmony::Error>),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotALobby(channel_id) => write!(f, "{} is not a lobby", channel_id.mention()),
            Self::AlreadyInQueue(user_id) => {
                write!(f, "{} is already in the queue", user_id.mention())
            }
            Self::NotInQueue(user_id) => {
                write!(f, "{} is not in the queue", user_id.mention())
            }
            Self::Harmony(err) => err.fmt(f),
        }
    }
}

impl From<(ChannelId, ErrorKind)> for Error {
    fn from((channel_id, kind): (ChannelId, ErrorKind)) -> Self {
        Self { channel_id, kind }
    }
}

impl From<harmony::Error> for ErrorKind {
    fn from(error: harmony::Error) -> Self {
        Self::Harmony(Box::new(error))
    }
}

fn f(msg: &str) -> Option<(&str, Vec<&str>)> {
    let mut it = msg.split_whitespace();
    let command = it.next()?;
    Some((command, it.collect()))
}

fn ready(ctx: Context, _: Ready, lobbies: Arc<Mutex<Lobbies>>) -> Result<(), Error> {
    println!("Bot started");
    thread::spawn(move || {
        let duration = chrono::Duration::minutes(1);
        loop {
            thread::sleep(Duration::from_secs(1));
            let now = Utc::now();
            for (channel_id, lobby) in lobbies.lock().iter_mut() {
                let inactives = lobby
                    .players
                    .iter()
                    .filter_map(|(&user_id, &date_time)| {
                        if now - date_time > duration {
                            Some(user_id)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                for user_id in inactives {
                    if lobby.remove(&user_id).is_some() {
                        ctx.send_message(*channel_id, |m| {
                            m.embed(|e| {
                                e.description(format!(
                                    "[{}/{}] {} has left the queue",
                                    lobby.len(),
                                    2 * lobby.size,
                                    user_id.mention()
                                ))
                            })
                        })
                        .unwrap();
                    }
                }
            }
        }
    });
    Ok(())
}

fn message_create(ctx: Context, msg: Message, lobbies: Arc<Mutex<Lobbies>>) -> Result<(), Error> {
    let fs = HashMap::from([
        (
            "!ping",
            Box::new(|_| {
                ping(&ctx, &msg);
                Ok(())
            }) as Box<dyn Fn(_) -> _>,
        ),
        ("!join", Box::new(|_| join(&ctx, &msg, &mut lobbies.lock()))),
        (
            "!leave",
            Box::new(|_| leave(&ctx, &msg, &mut lobbies.lock())),
        ),
    ]);
    if let Some((command, args)) = f(&msg.content) {
        if let Some(f) = fs.get(&command) {
            f(args)?;
        }
    }
    Ok(())
}

fn ping(ctx: &Context, msg: &Message) {
    let ctx = ctx.clone();
    let channel_id = msg.channel_id;
    let timestamp = msg.timestamp;
    thread::spawn(move || {
        let reply = ctx.send_message(channel_id, |m| m.content("pong"));
        // println!("{:#?}", reply);
        let reply = reply.unwrap();
        let duration = reply.timestamp - timestamp;
        ctx.edit_message(&reply, |m| {
            m.content(format!(
                "Pong! That took {} ms.",
                duration.num_milliseconds()
            ))
        })
        .unwrap();
    });
}

fn join(ctx: &Context, msg: &Message, lobbies: &mut Lobbies) -> Result<(), Error> {
    if let Some(lobby) = lobbies.get_mut(&msg.channel_id) {
        if lobby.insert(msg.author.id, msg.timestamp).is_none() {
            ctx.send_message(msg.channel_id, |m| {
                m.embed(|e| {
                    e.description(format!(
                        "[{}/{}] {} joined the queue.",
                        lobby.len(),
                        2 * lobby.size,
                        msg.author.id.mention()
                    ))
                })
            })
            .map_err(|err| Error {
                channel_id: msg.channel_id,
                kind: err.into(),
            })?;
        } else {
            return Err(Error {
                channel_id: msg.channel_id,
                kind: ErrorKind::AlreadyInQueue(msg.author.id),
            });
        }
    } else {
        return Err(Error {
            channel_id: msg.channel_id,
            kind: ErrorKind::NotALobby(msg.channel_id),
        });
    }
    Ok(())
}

fn leave(ctx: &Context, msg: &Message, lobbies: &mut Lobbies) -> Result<(), Error> {
    if let Some(lobby) = lobbies.get_mut(&msg.channel_id) {
        if lobby.remove(&msg.author.id).is_some() {
            ctx.send_message(msg.channel_id, |m| {
                m.embed(|e| {
                    e.description(format!(
                        "[{}/{}] {} left the queue",
                        lobby.len(),
                        2 * lobby.size,
                        msg.author.id.mention()
                    ))
                })
            })
            .map_err(|err| Error {
                channel_id: msg.channel_id,
                kind: err.into(),
            })?;
        } else {
            return Err(Error {
                channel_id: msg.channel_id,
                kind: ErrorKind::NotInQueue(msg.author.id),
            });
        }
    } else {
        return Err(Error {
            channel_id: msg.channel_id,
            kind: ErrorKind::NotALobby(msg.channel_id),
        });
    }
    Ok(())
}

fn main() {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN");
    let lobbies = Arc::new(Mutex::new(Lobbies::new()));
    lobbies
        .lock()
        .insert(794652629325447228.into(), Lobby::new(4));
    let client = ClientBuilder::new()
        .with_bot_token(&token)
        .intents(Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES)
        .on_ready(|ctx, rdy| {
            ready(ctx, rdy, lobbies.clone()).unwrap();
        })
        .on_message_create(|ctx, msg| {
            message_create(ctx, msg, lobbies.clone()).unwrap();
        })
        .build();
    if let Err(err) = client.run() {
        println!("Error: {}", err);
    }
}
