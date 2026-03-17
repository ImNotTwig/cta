use std::sync::Arc;

use anyhow::Error;
use lavalink_rs::prelude::PlayerContext;
use poise::serenity_prelude::{ChannelId, GuildId};
use thiserror::Error;

use crate::{Context, util::reply_no_ping};

mod insert;
mod join;
mod leave;
mod nowplaying;
mod pause;
mod play;
mod playnext;
mod queue;
mod skip;

pub use insert::insert;
pub use join::join;
pub use leave::leave;
pub use nowplaying::nowplaying;
pub use pause::pause;
pub use play::play;
pub use playnext::playnext;
pub use queue::queue;
pub use skip::skip;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum VoiceError {
    #[error("already in a voice channel for this guild")]
    AlreadyInVoice,
    #[error("user not in a voice channel")]
    UserNotInVoice,
    #[error("unknown")]
    Unknown,
}

/// Join a voice channel.
pub async fn join_vc(
    ctx: &Context<'_>,
    guild_id: GuildId,
    channel_id: Option<ChannelId>,
) -> anyhow::Result<(), Error> {
    let user_id = ctx.cache().current_user().id;
    let no_vc = match ctx.http().get_user_voice_state(guild_id, user_id).await {
        Err(_) => true,
        Ok(inner) => inner.channel_id.is_none(),
    };

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if no_vc || manager.get(guild_id).is_none() {
        let connect_to = match channel_id {
            None => {
                let user_channel_id = ctx.guild().and_then(|x| {
                    x.voice_states
                        .get(&ctx.author().id)
                        .and_then(|vs| vs.channel_id)
                });

                match user_channel_id {
                    Some(channel) => channel,
                    None => {
                        reply_no_ping(*ctx, "You are not in the voice channel hoe").await?;

                        anyhow::bail!(VoiceError::UserNotInVoice);
                    }
                }
            }
            Some(x) => x,
        };

        let lavalink = ctx.data().lavalink.clone();
        let voice_handler = manager.join_gateway(guild_id, connect_to).await?;

        if lavalink.get_player_context(guild_id).is_none() {
            lavalink
                .create_player_context_with_data(
                    guild_id,
                    voice_handler.0,
                    Arc::new((ctx.channel_id(), ctx.serenity_context().http.clone())),
                )
                .await?;
        }

        reply_no_ping(*ctx, format!("Joined: <#{}>!", connect_to)).await?;
        return Ok(());
    }

    anyhow::bail!(VoiceError::AlreadyInVoice)
}

pub async fn skip_stuck(player: PlayerContext) -> anyhow::Result<()> {
    let queue = player.get_queue();
    if let Ok(player_data) = player.get_player().await
        && player_data.track.is_none()
        && queue.get_track(0).await.is_ok_and(|x| x.is_some())
    {
        player.skip()?;
    }
    Ok(())
}
