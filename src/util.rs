use lavalink_rs::model::track::TrackData;
use poise::{
    CreateReply,
    serenity_prelude::{CreateAllowedMentions, GuildId},
};

use crate::Context;

pub async fn reply_no_ping<T: Into<String>>(ctx: Context<'_>, str: T) -> anyhow::Result<()> {
    let reply = ctx
        .reply_builder(
            CreateReply::default()
                .allowed_mentions(CreateAllowedMentions::default().replied_user(false))
                .reply(true),
        )
        .content(str.into());
    ctx.send(reply).await?;
    Ok(())
}

pub async fn in_same_vc(ctx: Context<'_>, guild_id: GuildId) -> anyhow::Result<bool> {
    let my_id = ctx.cache().current_user().id;
    let author_id = ctx.author().id;
    let my_vc = ctx
        .http()
        .get_user_voice_state(guild_id, my_id)
        .await?
        .channel_id;
    let author_vc = ctx
        .http()
        .get_user_voice_state(guild_id, author_id)
        .await?
        .channel_id;
    if my_vc == author_vc {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Check if the author in this context has moderate_members permission
/// This function will panic under a non-guild context
pub async fn is_moderator(ctx: Context<'_>) -> bool {
    let member = ctx.author_member().await.unwrap();
    ctx.guild()
        .unwrap()
        .member_permissions(&member)
        .moderate_members()
}

pub fn format_song(track: TrackData) -> String {
    if let Some(uri) = track.info.uri {
        format!("[{} - {}](<{uri}>)", track.info.author, track.info.title)
    } else {
        format!("{} - {}", track.info.author, track.info.title)
    }
}
