use std::{future::Future, pin::Pin, sync::Arc};

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{parser::CommandWithData, State};

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
