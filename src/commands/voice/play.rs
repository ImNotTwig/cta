use anyhow::Error;
use lavalink_rs::prelude::{TrackInQueue, TrackLoadData};

use crate::{
    Context,
    commands::{VoiceError, join_vc, skip_stuck},
    util::{format_song, in_same_vc, reply_no_ping},
};

/// Play a song in the voice channel you are connected in. Or resume the current track if paused.
#[poise::command(slash_command, prefix_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Search term or URL"]
    #[rest]
    term: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    match join_vc(&ctx, guild_id, None).await {
        Err(err) => match err.downcast_ref::<VoiceError>() {
            Some(VoiceError::AlreadyInVoice) => {
                let in_same_vc = match in_same_vc(ctx, guild_id).await {
                    Ok(val) => val,
                    _ => false,
                };
                if !in_same_vc {
                    ctx.reply("You cannot play music on the bot if you are not in the same voice channel. Fuck you.").await?;
                }
            }
            _ => return Err(err),
        },
        _ => {}
    };

    let lavalink = ctx.data().lavalink.clone();
    let player = match lavalink.get_player_context(guild_id) {
        None => {
            tracing::error!(
                "Somehow Couldn't get player_context for {}, even though join_vc didn't fail.",
                guild_id
            );
            ctx.reply("An error has occurred that shouldn't have happened. <@389202953862512641>")
                .await?;
            anyhow::bail!(VoiceError::Unknown);
        }
        Some(p) => p,
    };

    let query = if let Some(term) = term {
        if url::Url::parse(&term).is_ok() || term.starts_with("ytsearch:") {
            term
        } else {
            format!("{}:{}", "qbsearch", &term)
        }
    } else {
        player.set_pause(false).await?;
        reply_no_ping(ctx, "Resumed.").await?;
        return Ok(());
    };
    let loaded_tracks = lavalink.load_tracks(guild_id, &query).await?;
    let mut playlist_info = None;
    let mut tracks: Vec<TrackInQueue> = match loaded_tracks.data {
        Some(TrackLoadData::Track(x)) => vec![x.into()],
        Some(TrackLoadData::Search(x)) => vec![x[0].clone().into()],
        Some(TrackLoadData::Playlist(x)) => {
            playlist_info = Some(x.info);
            x.tracks.iter().map(|x| x.clone().into()).collect()
        }
        _ => {
            ctx.say(format!("Error: {:?}; <@389202953862512641>", loaded_tracks))
                .await?;
            return Ok(());
        }
    };
    if let Some(info) = playlist_info {
        reply_no_ping(ctx, format!("Added playlist to queue: {}", info.name)).await?;
    } else {
        let track = tracks[0].track.clone();

        reply_no_ping(ctx, format!("Added to queue: {}", format_song(track))).await?;
    }
    for i in &mut tracks {
        i.track.user_data = Some(serde_json::json!({"requester_id": ctx.author().id.get()}));
    }
    player.get_queue().append(tracks.into())?;
    skip_stuck(player).await?;
    Ok(())
}
