use anyhow::Error;

use crate::{
    Context,
    util::{in_same_vc, is_moderator},
};

/// Disconnects the bot from the voice channel it's currently in.
#[poise::command(slash_command, prefix_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let can_leave = match in_same_vc(ctx, guild_id).await {
            Ok(val) => val,
            _ => false,
        };

        if !can_leave && !is_moderator(ctx).await {
            ctx.reply("You cannot make the bot leave a voice channel that you're not in.")
                .await?;
            return Ok(());
        }

        let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
        ctx.data().lavalink.delete_player(guild_id).await?;
        manager.remove(guild_id).await?;
    }
    Ok(())
}
