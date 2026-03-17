use anyhow::Error;

use crate::{
    Context,
    util::{in_same_vc, is_moderator, reply_no_ping},
};

/// Pauses the currently playing track.
#[poise::command(slash_command, prefix_command)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let can_pause = match in_same_vc(ctx, guild_id).await {
            Ok(val) => val,
            _ => false,
        };

        if !can_pause && !is_moderator(ctx).await {
            ctx.reply("You cannot make the bot pause when you're not in the same voice channel.")
                .await?;
            return Ok(());
        }
        let lavalink = ctx.data().lavalink.get_player_context(guild_id).unwrap();

        if let Ok(_) = lavalink.set_pause(true).await {
            reply_no_ping(ctx, "Paused.").await?;
        } else {
            reply_no_ping(
                ctx,
                "There's nothing to pause, or there was an error while pausing.",
            )
            .await?;
        }
    }
    Ok(())
}
