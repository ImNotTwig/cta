use std::{future::Future, pin::Pin};

use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::{parser::CommandWithData, State};

async fn ping_impl(s: State, m: MessageCreate, c: CommandWithData) -> anyhow::Result<()> {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("time travel???????????????????????????")
        .as_secs();

    let mut content = None;
    if let Some(args) = c.arguments {
        if !args.is_empty() {
            content = args[0].string();
        }
    }

    s.http
        .create_message(m.channel_id)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .content(&format!(
            "{} - processed <t:{current_time}:R>",
            content.unwrap_or_else(|| String::from("Pong!"))
        ))
        .reply(m.id)
        .await?;
    Ok(())
}
pub fn ping(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    Box::pin(ping_impl(s, m, c))
}
