use lavalink_rs::{client::LavalinkClient, hook, model::events};
use poise::serenity_prelude::{ChannelId, CreateAllowedMentions, CreateMessage, Http};

use crate::util::format_song;

#[hook]
pub async fn track_start(client: LavalinkClient, _session_id: String, event: &events::TrackStart) {
    let player_context = client.get_player_context(event.guild_id).unwrap();
    let data = player_context
        .data::<(ChannelId, std::sync::Arc<Http>)>()
        .unwrap();
    let (channel_id, http) = (&data.0, &data.1);
    let msg = {
        let track = &event.track;
        format!(
            "Now playing: {} | Requested by <@{}>",
            format_song(track.clone()),
            track.user_data.clone().unwrap()["requester_id"]
        )
    };
    let builder = CreateMessage::default()
        .content(msg)
        .allowed_mentions(CreateAllowedMentions::new().all_users(false));
    let _ = channel_id.send_message(http, builder).await;
}
