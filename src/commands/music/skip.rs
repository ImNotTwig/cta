use std::{future::Future, pin::Pin};

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{parser::CommandWithData, State};

async fn next_impl(s: State, m: MessageCreate, _c: CommandWithData) -> anyhow::Result<()> {
    let mut lock = s.vcs.write().await;

    if let Some(queue) = lock.get_mut(&m.guild_id.unwrap()) {
        if queue.pos() >= queue.len() - 1 {
            return Ok(());
        }
        queue
            .goto(&s.songbird, &s.http, m.guild_id.unwrap(), queue.pos() + 1)
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
    let mut lock = s.vcs.write().await;

    if let Some(queue) = lock.get_mut(&m.guild_id.unwrap()) {
        if queue.pos() == 0 {
            return Ok(());
        }
        queue
            .goto(&s.songbird, &s.http, m.guild_id.unwrap(), queue.pos() - 1)
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
