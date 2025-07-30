use std::{future::Future, pin::Pin, sync::Arc};

use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::{parser::CommandWithData, state::Handler, State};

async fn prefix_impl(s: State, m: MessageCreate, c: CommandWithData) -> anyhow::Result<()> {
    let mut maybe_pfx = None;
    if let Some(args) = c.arguments {
        if !args.is_empty() {
            maybe_pfx = args[0].string();
        }
    }

    if let Some(pfx) = maybe_pfx {
        let mut configs = s.server_configs.lock().await;
        if let Some(config) = configs.get_mut(&m.guild_id.unwrap()) {
            config.set_prefix(&pfx);

            s.http
                .create_message(m.channel_id)
                .allowed_mentions(Some(&AllowedMentions::default()))
                .content(&format!("Changed prefix to `{pfx}`."))
                .reply(m.id)
                .await?;
        }
    }
    Arc::clone(&s).write_configs_to_file().await?;
    Ok(())
}
pub fn prefix(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    Box::pin(prefix_impl(s, m, c))
}
