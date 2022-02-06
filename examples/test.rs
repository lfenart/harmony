use std::env;

use harmony::client::{ClientBuilder, Context};
use harmony::gateway::{Intents, Ready};
use harmony::model::{Activity, Member, Message, Role};

const PREFIX: &str = "!";

fn ready(ctx: Context, _: Ready) {
    println!("Bot started");
    ctx.set_activity(Activity::playing("Star Wars Battlefront II"))
        .ok();
}

fn message_create(ctx: Context, msg: Message) {
    let content = if let Some(content) = msg.content.strip_prefix(PREFIX) {
        content
    } else {
        // The message does not start with the prefix
        return;
    };
    let mut args = content.split_whitespace();
    let command = args.next().unwrap_or_default();
    let args = args.collect::<Vec<_>>();
    let result = match command {
        "ping" => ping(&ctx, &msg),
        "newrole" => newrole(&ctx, &msg, &args),
        "deleterole" => deleterole(&ctx, &msg, &args),
        "giverole" => giverole(&ctx, &msg, &args),
        "removerole" => removerole(&ctx, &msg, &args),
        "delete" => delete(&ctx, &msg),
        _ => Ok(()),
    };
    if let Err(err) = result {
        eprintln!("{:?}", err);
    }
}

fn ping(ctx: &Context, msg: &Message) -> harmony::Result {
    let reply = ctx.send_message(msg.channel_id, |m| m.content("Pong!"))?;
    let duration = reply.timestamp - msg.timestamp;
    ctx.edit_message(reply.channel_id, reply.id, |m| {
        m.content(format!(
            "Pong! That took {} ms.",
            duration.num_milliseconds()
        ))
    })?;
    Ok(())
}

fn newrole(ctx: &Context, msg: &Message, args: &[&str]) -> harmony::Result {
    if args.is_empty() {
        ctx.send_message(msg.channel_id, |m| m.content("Not enough arguments"))?;
        return Ok(());
    }
    let guild_id = if let Some(guild_id) = msg.guild_id {
        guild_id
    } else {
        ctx.send_message(msg.channel_id, |m| m.content("Not in a guild"))?;
        return Ok(());
    };
    ctx.create_guild_role(guild_id, |r| r.name(&args[0]))?;
    Ok(())
}

fn deleterole(ctx: &Context, msg: &Message, args: &[&str]) -> harmony::Result {
    if args.is_empty() {
        ctx.send_message(msg.channel_id, |m| m.content("Not enough arguments"))?;
        return Ok(());
    }
    let guild_id = if let Some(guild_id) = msg.guild_id {
        guild_id
    } else {
        ctx.send_message(msg.channel_id, |m| m.content("Not in a guild"))?;
        return Ok(());
    };
    let role_id = if let Some(role) = ctx
        .get_guild_roles(guild_id)?
        .into_iter()
        .find(|x| x.name == args[0])
    {
        role.id
    } else {
        ctx.send_message(msg.channel_id, |m| m.content("Role not found"))?;
        return Ok(());
    };
    ctx.delete_guild_role(guild_id, role_id)?;
    Ok(())
}

fn giverole(ctx: &Context, msg: &Message, args: &[&str]) -> harmony::Result {
    if args.len() < 2 {
        ctx.send_message(msg.channel_id, |m| m.content("Not enough arguments"))?;
        return Ok(());
    }
    let guild_id = if let Some(guild_id) = msg.guild_id {
        guild_id
    } else {
        ctx.send_message(msg.channel_id, |m| m.content("Not in a guild"))?;
        return Ok(());
    };
    let user_id = if let Some(member) = Member::parse(ctx, guild_id, args[0])? {
        member.user.id
    } else {
        ctx.send_message(msg.channel_id, |m| m.content("Member not found"))?;
        return Ok(());
    };
    let role_id = if let Some(role) = Role::parse(ctx, guild_id, args[1])? {
        role.id
    } else {
        ctx.send_message(msg.channel_id, |m| m.content("Role not found"))?;
        return Ok(());
    };
    ctx.add_guild_member_role(guild_id, user_id, role_id)?;
    Ok(())
}

fn removerole(ctx: &Context, msg: &Message, args: &[&str]) -> harmony::Result {
    if args.len() < 2 {
        ctx.send_message(msg.channel_id, |m| m.content("Not enough arguments"))?;
        return Ok(());
    }
    let guild_id = if let Some(guild_id) = msg.guild_id {
        guild_id
    } else {
        ctx.send_message(msg.channel_id, |m| m.content("Not in a guild"))?;
        return Ok(());
    };
    let user_id = if let Some(member) = Member::parse(ctx, guild_id, args[0])? {
        member.user.id
    } else {
        ctx.send_message(msg.channel_id, |m| m.content("Member not found"))?;
        return Ok(());
    };
    let role_id = if let Some(role) = Role::parse(ctx, guild_id, args[1])? {
        role.id
    } else {
        ctx.send_message(msg.channel_id, |m| m.content("Role not found"))?;
        return Ok(());
    };
    ctx.remove_guild_member_role(guild_id, user_id, role_id)?;
    Ok(())
}

fn delete(ctx: &Context, msg: &Message) -> harmony::Result {
    ctx.delete_message(msg.channel_id, msg.id)?;
    Ok(())
}

fn main() {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN");
    let client = ClientBuilder::new()
        .with_bot_token(&token)
        .intents(Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES)
        .on_ready(ready)
        .on_message_create(message_create)
        .build();
    if let Err(err) = client.run() {
        println!("Error: {}", err);
    }
}
