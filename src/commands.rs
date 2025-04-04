use std::{future::Future, pin::Pin, random::DefaultRandomSource, sync::Arc};

use songbird::input::{Compose, YoutubeDl};
use std::random::Random;
use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::{music::Queue, parser::CommandWithData, State};

async fn ping_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let mut content;
    if let Some(args) = c.arguments {
        content = args[0].clone().string();
        if content == "" {
            content = String::from("pong!");
        }
    } else {
        content = String::from("pong!");
    };

    s.http
        .create_message(m.channel_id)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .content(&content)
        .reply(m.id)
        .await?;
    Ok(())
}
pub fn ping(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(ping_impl(sc, mc, cc)))(s, m, c);
}

async fn jump_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let mut amount = 1;
    if let Some(args) = c.arguments {
        if args.len() > 0 {
            amount = args[0].clone().uint();
        }
    };

    for i in 0..amount {
        let res = bool::random(&mut DefaultRandomSource);
        let content = if res {
            "YOU MADE THE JUMP!!! YOURE SO AWESOME. HERE'S THE BEEF."
        } else {
            "CONGRATULATIONS, YOU'VE DROPPED YOURSELF INTO NOOB JAIL, DUMBASS."
        };
        s.http
            .create_message(m.channel_id)
            .allowed_mentions(Some(&AllowedMentions::default()))
            .content(content)
            .reply(m.id)
            .await?;
    }
    Ok(())
}
pub fn jump(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(jump_impl(sc, mc, cc)))(s, m, c);
}

async fn join_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let vc = s
        .http
        .user_voice_state(m.guild_id.unwrap(), m.author.id)
        .await?
        .model()
        .await?
        .channel_id
        .unwrap();

    s.songbird.join(m.guild_id.unwrap(), vc).await?;

    s.http
        .create_message(m.channel_id)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .content(&format!("Joined: <#{}>", vc))
        .reply(m.id)
        .await?;
    Ok(())
}
pub fn join(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(join_impl(sc, mc, cc)))(s, m, c);
}

async fn leave_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    if let Some(call_lock) = s.songbird.get(m.guild_id.unwrap()) {
        let call = call_lock.lock().await;

        if let Some(channel) = call.current_channel() {
            let vc = s
                .http
                .user_voice_state(m.guild_id.unwrap(), m.author.id)
                .await?
                .model()
                .await?
                .channel_id
                .unwrap();

            if channel == vc.into() {
                s.songbird.leave(m.guild_id.unwrap()).await?;
                let mut lock = s.vcs.write().await;

                if let Some(queue) = lock.remove(&m.guild_id.unwrap()) {
                    queue.drop();
                }
                s.http
                    .create_message(m.channel_id)
                    .allowed_mentions(Some(&AllowedMentions::default()))
                    .content(&format!("Left: <#{}>, and cleared the queue.", vc))
                    .reply(m.id)
                    .await?;
            } else {
                s.http
                    .create_message(m.channel_id)
                    .allowed_mentions(Some(&AllowedMentions::default()))
                    .content(&format!("You are not in <#{}>. FUCK YOU!", vc))
                    .reply(m.id)
                    .await?;
            }
        }
    }

    Ok(())
}
pub fn leave(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(leave_impl(sc, mc, cc)))(s, m, c);
}

async fn play_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let guild_id = m
        .guild_id
        .expect("Cannot use the play command outside of a guild.");

    let mut url;
    if let Some(args) = c.arguments {
        url = args[0].clone().string();
        if url == "" {
            url = String::new();
        }
    } else {
        url = String::new();
    };

    if s.songbird.get(guild_id).is_none() {
        let vc = s
            .http
            .user_voice_state(m.guild_id.unwrap(), m.author.id)
            .await?
            .model()
            .await?
            .channel_id
            .unwrap();

        s.songbird.join(guild_id, vc).await?;
    }

    let mut src = YoutubeDl::new(s.client.clone(), url);
    if let Ok(meta) = src.aux_metadata().await {
        let content = format!(
            "Added: '{} - {}' to Queue",
            meta.title.unwrap(),
            meta.artist.unwrap()
        );

        s.http
            .create_message(m.channel_id)
            .content(&content)
            .await?;

        let mut lock = s.vcs.write().await;
        if lock.get(&guild_id).is_none() {
            lock.insert(
                guild_id,
                Queue::new(None, None, Some(m.channel_id), vec![src]),
            );
            lock.get_mut(&guild_id)
                .unwrap()
                .play(&s.songbird, &s.http, guild_id)
                .await?;
        } else {
            lock.get_mut(&guild_id).unwrap().push(src);
        }
    }

    Ok(())
}
pub fn play(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(play_impl(sc, mc, cc)))(s, m, c);
}

async fn next_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let mut lock = s.vcs.write().await;

    if let Some(queue) = lock.get_mut(&m.guild_id.unwrap()) {
        queue
            .next(&s.songbird, &s.http, m.guild_id.unwrap())
            .await?;
    }

    Ok(())
}

pub fn next(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(next_impl(sc, mc, cc)))(s, m, c);
}

async fn prev_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let mut lock = s.vcs.write().await;

    if let Some(queue) = lock.get_mut(&m.guild_id.unwrap()) {
        queue
            .prev(&s.songbird, &s.http, m.guild_id.unwrap())
            .await?;
    }

    Ok(())
}

pub fn prev(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(prev_impl(sc, mc, cc)))(s, m, c);
}

async fn unpause_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let mut lock = s.vcs.write().await;

    if let Some(queue) = lock.get_mut(&m.guild_id.unwrap()) {
        queue.unpause().await?;
    }

    Ok(())
}

pub fn unpause(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(unpause_impl(sc, mc, cc)))(s, m, c);
}

async fn pause_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let mut lock = s.vcs.write().await;

    if let Some(queue) = lock.get_mut(&m.guild_id.unwrap()) {
        queue.pause().await?;
    }

    Ok(())
}

pub fn pause(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(pause_impl(sc, mc, cc)))(s, m, c);
}
