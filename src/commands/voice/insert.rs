use anyhow::Error;
use lavalink_rs::prelude::{TrackInQueue, TrackLoadData};

use crate::{
    Context,
    util::{format_song, in_same_vc, is_moderator, reply_no_ping},
};

/// Insert a track into the queue at the designated position.
#[poise::command(slash_command, prefix_command)]
pub async fn insert(
    ctx: Context<'_>,
    #[description = "Index of insertion"] index: usize,
    #[description = "Search term or URL"]
    #[rest]
    term: Option<String>,
) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let can_insert = match in_same_vc(ctx, guild_id).await {
            Ok(val) => val,
            _ => false,
        };

        if !can_insert && !is_moderator(ctx).await {
            ctx.reply(
                "You cannot insert into the queue when you're not in the same voice channel as the bot.",
            )
            .await?;
            return Ok(());
        }

        if index == 0 {
            reply_no_ping(ctx, "Cannot insert track at 0th position. (use playnext)").await?;
            return Ok(());
        }

        let query = if let Some(term) = term {
            if url::Url::parse(&term).is_ok() || term.starts_with("ytsearch:") {
                term
            } else {
                format!("{}:{}", "qbsearch", &term)
            }
        } else {
            reply_no_ping(ctx, "Cannot insert `NOTHING` into the queue.").await?;
            return Ok(());
        };

        let lavalink_client = ctx.data().lavalink.clone();
        let player = lavalink_client.get_player_context(guild_id).unwrap();

        let loaded_tracks = lavalink_client.load_tracks(guild_id, &query).await?;
        let mut track: TrackInQueue = match loaded_tracks.data {
            Some(TrackLoadData::Track(x)) => x.into(),
            Some(TrackLoadData::Search(x)) => x[0].clone().into(),
            Some(TrackLoadData::Playlist(x)) => {
                // We only want to add the first track of the playlist,
                // because inserting a whole playlist into the middle of the queue is stupid.
                x.tracks[0].clone().into()
            }
            _ => {
                ctx.say(format!("Error: {:?}; <@389202953862512641>", loaded_tracks))
                    .await?;
                return Ok(());
            }
        };

        track.track.user_data = Some(serde_json::json!({"requester_id": ctx.author().id.get()}));
        reply_no_ping(
            ctx,
            format!(
                "Inserted: {} at position: {}",
                format_song(track.track.clone()),
                index
            ),
        )
        .await?;
        player.get_queue().insert(index - 1, track.track)?;
    }
    Ok(())
}
