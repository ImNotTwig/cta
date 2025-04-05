use std::{future::Future, pin::Pin, sync::Arc};

use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::{parser::CommandWithData, State};

async fn queue_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let content;
    if let Some(queue) = s.vcs.read().await.get(&m.guild_id.unwrap()) {
        content = queue.get_tracklist().await;

        let mut str = String::from("Queue\n```\n");
        let mut i = 0;
        for j in content.iter() {
            if queue.pos() == i {
                str += &(String::from("-> ") + j + "\n");
            } else {
                str += &(format!("{}: ", i + 1) + j + "\n");
            }
            i += 1;
        }
        str += "```";

        s.http
            .create_message(m.channel_id)
            .allowed_mentions(Some(&AllowedMentions::default()))
            .content(&format!("{str}"))
            .reply(m.id)
            .await?;
    } else {
        return Ok(());
    };

    Ok(())
}
pub fn queue(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(queue_impl(sc, mc, cc)))(s, m, c);
}
