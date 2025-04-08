use std::{future::Future, pin::Pin, sync::Arc};

use tokio::sync::Mutex;
use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::{music::Queue, parser::CommandWithData, State};

async fn join_impl(s: State, m: MessageCreate, _c: CommandWithData) -> anyhow::Result<()> {
    let vc = s
        .http
        .user_voice_state(m.guild_id.unwrap(), m.author.id)
        .await?
        .model()
        .await?
        .channel_id
        .unwrap();

    s.songbird.join(m.guild_id.unwrap(), vc).await?;
    let mut lock = s.vcs.write().await;
    if lock.get(&m.guild_id.unwrap()).is_none() {
        lock.insert(
            m.guild_id.unwrap(),
            Arc::new(Mutex::new(Queue::new(None, None, Some(m.channel_id)))),
        );
    }

    s.http
        .create_message(m.channel_id)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .content(&format!("Joined: <#{}>", vc))
        .reply(m.id)
        .await?;
    Ok(())
}
pub fn join(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(join_impl(sc, mc, cc)))(s, m, c);
}

async fn leave_impl(s: State, m: MessageCreate, _c: CommandWithData) -> anyhow::Result<()> {
    if let Some(call_lock) = s.songbird.get(m.guild_id.unwrap()) {
        let mut call = call_lock.lock().await;

        if let Some(channel) = call.current_channel() {
            let vc = s
                .http
                .user_voice_state(m.guild_id.unwrap(), m.author.id)
                .await?
                .model()
                .await?
                .channel_id
                .unwrap();

            call.leave().await?;

            let mut lock = s.vcs.write().await;
            lock.remove(&m.guild_id.unwrap()).unwrap();

            if channel == vc.into() {
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
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(leave_impl(sc, mc, cc)))(s, m, c);
}
