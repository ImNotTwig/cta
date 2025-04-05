use std::{
    future::Future,
    pin::Pin,
    random::{DefaultRandomSource, Random},
    sync::Arc,
};

use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::{parser::CommandWithData, State};

async fn jump_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let mut amount = 1;
    if let Some(args) = c.arguments {
        if args.len() > 0 {
            amount = args[0].clone().uint();
        }
    };

    if amount <= 10 {
        for i in 0..amount {
            let res = bool::random(&mut DefaultRandomSource);
            let content = if res {
                "YOU MADE THE JUMP!!! YOURE SO AWESOME. HERE'S THE BEEF."
            } else {
                "CONGRATULATIONS, YOU'VE DROPPED YOURSELF INTO NOOB JAIL, DUMBASS."
            };
            s.http
                .create_message(m.channel_id)
                .allowed_mentions(Some(&AllowedMentions::default()))
                .content(content)
                .reply(m.id)
                .await?;
        }
    } else {
        s.http
            .create_message(m.channel_id)
            .allowed_mentions(Some(&AllowedMentions::default()))
            .content("YOU TOOK THE FIZZY LIFTING DRINKS. YOU LOSE. GOOD DAY TO YOU.")
            .reply(m.id)
            .await?;
    }
    Ok(())
}
pub fn jump(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(jump_impl(sc, mc, cc)))(s, m, c);
}
