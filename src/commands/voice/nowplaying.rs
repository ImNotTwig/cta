use anyhow::Error;
use poise::{
    CreateReply,
    serenity_prelude::{Colour, CreateAllowedMentions, CreateEmbed},
};

use crate::{Context, util::reply_no_ping};

/// Show the currently playing track and its position/duration.
#[poise::command(slash_command, prefix_command)]
pub async fn nowplaying(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let lavalink = ctx.data().lavalink.get_player_context(guild_id).unwrap();

        if let Some(now_playing) = lavalink.get_player().await?.track {
            let pos = lavalink.get_player().await?.state.position;

            let current_secs = (pos / 1000) % 60;
            let current_mins = (pos / 1000) / 60;

            // Calculate total length
            let length_secs = (now_playing.info.length / 1000) % 60;
            let length_mins = (now_playing.info.length / 1000) / 60;

            let timestamp = format!(
                "{:02}:{:02}/{:02}:{:02}",
                current_mins, current_secs, length_mins, length_secs
            );

            let msg = CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .url(now_playing.info.uri.unwrap_or_else(|| String::from("")))
                        .color(Colour::new(0xf8c8dc))
                        .title(format!(
                            "{} - {}",
                            now_playing.info.author, now_playing.info.title
                        ))
                        .description(timestamp),
                )
                .reply(true)
                .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

            ctx.send(msg).await?;
        } else {
            reply_no_ping(ctx, "There's nothing playing.").await?;
        }
    }
    Ok(())
}
