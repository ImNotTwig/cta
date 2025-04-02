use std::{future::Future, pin::Pin, sync::Arc};

use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::parser::Command;

async fn ping_impl(h: Arc<HttpClient>, m: MessageCreate, c: CommandWithData) -> anyhow::Result<()> {

    h.create_message(m.channel_id)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .content("pong!")
        .reply(m.id)
        .await?;
    Ok(())
}
pub fn ping(
    h: Arc<HttpClient>,
    m: MessageCreate,
    c: Command,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |hc, mc, cc| Box::pin(ping_impl(hc, mc, cc)))(h, m, c);
}
