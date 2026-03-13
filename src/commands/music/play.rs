use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::Mutex;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{music::Queue, parser::CommandWithData, State};

async fn play_impl(s: State, m: MessageCreate, c: CommandWithData) -> anyhow::Result<()> {
    let guild_id = m
        .guild_id
        .expect("Cannot use the play command outside of a guild.");

    let mut url = None;
    if let Some(args) = c.arguments {
        if !args.is_empty() {
            url = args[0].clone().string();
        }
    }

    if s.songbird.get(guild_id).is_none() {
        let vc = s
            .http
            .user_voice_state(guild_id, m.author.id)
            .await?
            .model()
            .await?
            .channel_id
            .unwrap();

        s.songbird.join(guild_id, vc).await?;
    }

    let mut lock = s.vcs.lock().await;
    if let Some(queue_lock) = lock.get_mut(&guild_id) {
        let mut queue = queue_lock.lock().await;
        if let Some(u) = url {
            let meta = queue.push(Arc::clone(&s), u).await?;

            let content = format!(
                "Added: '{} - {}' to Queue",
                meta.artist.unwrap_or_else(|| String::from("UNKNOWN")),
                meta.title.unwrap_or_else(|| String::from("UNKNOWN")),
            );

            s.http
                .create_message(m.channel_id)
                .content(&content)
                .await?;
        }
        drop(queue);
    } else {
        lock.insert(
            guild_id,
            Arc::new(Mutex::new(Queue::new(None, None, Some(m.channel_id)))),
        );
        let queue_lock = lock.get_mut(&guild_id).unwrap();
        let mut queue = queue_lock.lock().await;
        _ = queue.push(Arc::clone(&s), url.unwrap()).await?;
        queue.play(Arc::clone(&s), guild_id).await?;
        drop(queue);
    }
    drop(lock);

    Ok(())
}
pub fn play(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    Box::pin(play_impl(s, m, c))
}
