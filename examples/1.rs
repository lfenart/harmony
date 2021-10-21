use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use harmony::client::{ClientBuilder, Context};
use harmony::gateway::Ready;
use harmony::model::id::{ChannelId, UserId};
use harmony::model::Message;

fn ready(_: &Context, _: &Ready, lobbies: Arc<Mutex<HashMap<ChannelId, HashSet<UserId>>>>) {
    println!("Bot started");
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        {
            let _x = lobbies.lock();
        }
    });
}

fn message_create(
    ctx: &Context,
    msg: &Message,
    lobbies: &mut HashMap<ChannelId, HashSet<UserId>>,
) -> harmony::Result {
    if msg.content.starts_with("!ping") {
        ping(ctx, msg);
        return Ok(());
    }
    if msg.content.starts_with("!join") {
        return join(ctx, msg, lobbies);
    }
    if msg.content.starts_with("!leave") {
        return leave(ctx, msg, lobbies);
    }
    Ok(())
}

fn ping(ctx: &Context, msg: &Message) {
    let ctx = ctx.clone();
    let channel_id = msg.channel_id;
    let timestamp = msg.timestamp;
    thread::spawn(move || {
        let reply = ctx.send_message(channel_id, |m| m.content("pong")).unwrap();
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

fn join(
    ctx: &Context,
    msg: &Message,
    lobbies: &mut HashMap<ChannelId, HashSet<UserId>>,
) -> harmony::Result {
    if let Some(lobby) = lobbies.get_mut(&msg.channel_id) {
        if lobby.insert(msg.author.id) {
            ctx.send_message(msg.channel_id, |m| {
                m.embed(|e| {
                    e.description(format!(
                        "[{}/8] {} joined the queue.",
                        lobby.len(),
                        msg.author.id.mention()
                    ))
                })
            })?;
        } else {
            ctx.send_message(msg.channel_id, |m| {
                m.embed(|e| {
                    e.description(format!(
                        "{} is already in the queue.",
                        msg.author.id.mention()
                    ))
                })
            })?;
        }
    } else {
        ctx.send_message(msg.channel_id, |m| {
            m.embed(|e| {
                e.description(format!("{} is not a lobby.", msg.channel_id.mention()))
                    .timestamp(chrono::Utc::now())
            })
        })?;
    }
    Ok(())
}

fn leave(
    ctx: &Context,
    msg: &Message,
    lobbies: &mut HashMap<ChannelId, HashSet<UserId>>,
) -> harmony::Result {
    if let Some(lobby) = lobbies.get_mut(&msg.channel_id) {
        if lobby.remove(&msg.author.id) {
            ctx.send_message(msg.channel_id, |m| {
                m.embed(|e| {
                    e.description(format!(
                        "[{}/8] {} left the queue",
                        lobby.len(),
                        msg.author.id.mention()
                    ))
                })
            })?;
        } else {
            ctx.send_message(msg.channel_id, |m| {
                m.embed(|e| {
                    e.description(format!("{} is not in the queue.", msg.author.id.mention()))
                })
            })?;
        }
    } else {
        ctx.send_message(msg.channel_id, |m| {
            m.embed(|e| e.description(format!("{} is not a lobby.", msg.channel_id.mention())))
        })?;
    }
    Ok(())
}

fn main() {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN");
    let lobbies = Arc::new(Mutex::new(HashMap::new()));
    lobbies
        .lock()
        .unwrap()
        .insert(794652629325447228.into(), HashSet::default());
    let client = ClientBuilder::new()
        .with_bot_token(&token)
        .on_ready(|ctx, rdy| {
            ready(ctx, rdy, lobbies.clone());
            Ok(())
        })
        .on_message_create(|ctx, msg| message_create(ctx, msg, &mut lobbies.lock().unwrap()))
        .build();
    client.start().unwrap();
}
