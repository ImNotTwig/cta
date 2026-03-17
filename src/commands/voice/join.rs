use anyhow::Error;
use poise::serenity_prelude::ChannelId;

use crate::{Context, commands::join_vc};

/// Join the specified voice channel or the one you are currently in.
#[poise::command(slash_command, prefix_command)]
pub async fn join(
    ctx: Context<'_>,
    #[description = "The channel ID to join to."]
    #[channel_types("Voice")]
    channel_id: Option<ChannelId>,
) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        join_vc(&ctx, guild_id, channel_id).await?;
    } else {
        ctx.reply("You cant make a Discord bot join a personal call.")
            .await?;
        return Ok(());
    }
    Ok(())
}
