use std::{future::Future, pin::Pin};

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{parser::CommandWithData, State};

async fn unpause_impl(s: State, m: MessageCreate, _c: CommandWithData) -> anyhow::Result<()> {
    let lock = s.vcs.lock().await.clone();
    if let Some(queue_lock) = lock.get(&m.guild_id.unwrap()) {
        let queue = queue_lock.lock().await;
        queue.unpause()?;
    }

    Ok(())
}

pub fn unpause(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(unpause_impl(sc, mc, cc)))(s, m, c);
}

async fn pause_impl(s: State, m: MessageCreate, _c: CommandWithData) -> anyhow::Result<()> {
    let vcs = s.vcs.lock().await.clone();
    if let Some(queue_lock) = vcs.get(&m.guild_id.unwrap()) {
        let queue = queue_lock.lock().await;
        queue.pause()?;
    }

    Ok(())
}

pub fn pause(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(pause_impl(sc, mc, cc)))(s, m, c);
}
