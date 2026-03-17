use anyhow::Error;
use poise::{
    CreateReply,
    serenity_prelude::{Colour, CreateEmbed, futures::StreamExt},
};

use crate::{Context, util::format_song};

/// Lists the tracks in this server's Queue
#[poise::command(slash_command, prefix_command)]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let lavalink = ctx.data().lavalink.get_player_context(guild_id).unwrap();

        let current_track = lavalink.get_player().await?.track.unwrap();

        let tracks = lavalink
            .get_queue()
            .enumerate()
            .map(|(index, x)| {
                let mut format = format_song(x.track.clone());
                if x == current_track.clone().into() {
                    format = format!("> {format}");
                } else {
                    format = format!("{index}. {format}");
                }
                format.push('\n');
                format
            })
            .collect::<String>()
            .await
            .trim()
            .to_string();

        let msg = CreateReply::default().embed(
            CreateEmbed::new()
                .color(Colour::new(0xf8c8dc))
                .title(format!("The Queue for {}", ctx.guild().unwrap().name))
                .description(tracks),
        );

        ctx.send(msg).await?;
    }
    Ok(())
}
