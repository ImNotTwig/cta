use songbird::input::{Compose, YoutubeDl};

use std::{future::Future, pin::Pin, sync::Arc};
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{music::Queue, parser::CommandWithData, State};

async fn play_impl(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> anyhow::Result<()> {
    let guild_id = m
        .guild_id
        .expect("Cannot use the play command outside of a guild.");

    let mut url;
    if let Some(args) = c.arguments {
        url = args[0].clone().string();
        if url == "" {
            url = String::new();
        }
    } else {
        url = String::new();
    };

    if s.songbird.get(guild_id).is_none() {
        let vc = s
            .http
            .user_voice_state(m.guild_id.unwrap(), m.author.id)
            .await?
            .model()
            .await?
            .channel_id
            .unwrap();

        s.songbird.join(guild_id, vc).await?;
    }

    let mut src = YoutubeDl::new(s.client.clone(), url.clone());
    let meta = match src.aux_metadata().await {
        Ok(meta) => meta,
        Err(_) => {
            src = YoutubeDl::new_search(s.client.clone(), url.clone());
            src.aux_metadata().await?
        }
    };

    let mut lock = s.vcs.write().await;
    if lock.get(&guild_id).is_none() {
        lock.insert(
            guild_id,
            Queue::new(None, None, Some(m.channel_id), vec![src]),
        );
    } else {
        lock.get_mut(&guild_id).unwrap().push(src);
    }

    if lock.len() == 1 {
        lock.get_mut(&guild_id)
            .unwrap()
            .play(&s.songbird, &s.http, guild_id)
            .await?;
    } else {
        let content = format!(
            "Added: '{} - {}' to Queue",
            meta.title.unwrap(),
            meta.artist.unwrap()
        );

        s.http
            .create_message(m.channel_id)
            .content(&content)
            .await?;
    }

    Ok(())
}
pub fn play(
    s: Arc<State<'static>>,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(play_impl(sc, mc, cc)))(s, m, c);
}
