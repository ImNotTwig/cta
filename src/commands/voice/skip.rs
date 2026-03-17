use anyhow::Error;

use crate::{
    Context,
    util::{format_song, in_same_vc, is_moderator, reply_no_ping},
};

/// Skips the current track.
#[poise::command(slash_command, prefix_command)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let can_skip = match in_same_vc(ctx, guild_id).await {
            Ok(val) => val,
            _ => false,
        };

        if !can_skip && !is_moderator(ctx).await {
            ctx.reply("You cannot make the bot skip when you're not in the same voice channel.")
                .await?;
            return Ok(());
        }
        let lavalink = ctx.data().lavalink.get_player_context(guild_id).unwrap();

        if let Some(now_playing) = lavalink.get_player().await?.track {
            reply_no_ping(ctx, format!("Skipped: {}", format_song(now_playing))).await?;
            lavalink.skip()?;
        } else {
            reply_no_ping(ctx, "There's nothing to skip.").await?;
        }
    }
    Ok(())
}
