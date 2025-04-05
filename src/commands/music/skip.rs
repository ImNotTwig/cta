use std::{future::Future, pin::Pin, sync::Arc};

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{parser::CommandWithData, State};

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
