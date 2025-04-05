use std::{future::Future, pin::Pin, sync::Arc};

use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::{parser::CommandWithData, State};

async fn ping_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("time travel???????????????????????????")
        .as_secs();

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
        .content(&format!("{content} - processed <t:{current_time}:R>"))
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
