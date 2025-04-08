use std::{future::Future, pin::Pin, sync::Arc};

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{parser::CommandWithData, State};

async fn next_impl(s: State, m: MessageCreate, _c: CommandWithData) -> anyhow::Result<()> {
    let mut lock = s.vcs.write().await.clone();

    if let Some(queue_lock) = lock.get_mut(&m.guild_id.unwrap()) {
        let mut queue = queue_lock.lock().await;
        let pos = queue.pos();
        if pos + 1 >= queue.len() {
            let call_lock = s.songbird.get(m.guild_id.unwrap()).unwrap();
            queue.stop(&mut call_lock.lock().await);
            return Ok(());
        }
        queue
            .goto(Arc::clone(&s), m.guild_id.unwrap(), pos + 1)
            .await?;
    }

    Ok(())
}

pub fn next(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(next_impl(sc, mc, cc)))(s, m, c);
}

async fn prev_impl(s: State, m: MessageCreate, _c: CommandWithData) -> anyhow::Result<()> {
    let mut lock = s.vcs.write().await.clone();

    if let Some(queue_lock) = lock.get_mut(&m.guild_id.unwrap()) {
        let mut queue = queue_lock.lock().await;
        let pos = queue.pos();
        if pos == 0 {
            return Ok(());
        }
        queue
            .goto(Arc::clone(&s), m.guild_id.unwrap(), pos - 1)
            .await?;
    }

    Ok(())
}

pub fn prev(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(prev_impl(sc, mc, cc)))(s, m, c);
}
