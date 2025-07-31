use std::time::Duration;

use songbird::{
    input::{AuxMetadata, Compose, YoutubeDl},
    tracks::TrackHandle,
};
use tokio::sync::MutexGuard;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::state::State;

#[derive(Clone)]
pub struct Queue<'a> {
    current_track: Option<TrackHandle>,
    current_track_len: Option<Duration>,
    songs: Vec<YoutubeDl<'a>>,
    pos: usize,
    text_channel: Option<Id<ChannelMarker>>,
}

impl Queue<'static> {
    pub const fn new(
        track: Option<TrackHandle>,
        track_len: Option<Duration>,
        text_channel: Option<Id<ChannelMarker>>,
    ) -> Self {
        Queue {
            current_track: track,
            current_track_len: track_len,
            songs: vec![],
            pos: 0,
            text_channel,
        }
    }

    pub async fn format_song(mut yt: YoutubeDl<'static>) -> String {
        let meta = yt.aux_metadata().await.expect("no metadata");
        format!(
            "'{} - {}'",
            meta.artist.unwrap_or_else(|| String::from("UNKNOWN")),
            meta.title.unwrap_or_else(|| String::from("UNKNOWN")),
        )
    }

    async fn set_current_track(
        &mut self,
        src: &mut YoutubeDl<'static>,
        call: &mut MutexGuard<'_, songbird::Call>,
    ) -> anyhow::Result<()> {
        self.current_track_len = src.aux_metadata().await?.duration;
        self.current_track = Some(call.play_input(src.clone().into()));
        Ok(())
    }

    pub const fn len(&self) -> usize {
        self.songs.len()
    }

    pub const fn pos(&self) -> usize {
        self.pos
    }

    pub async fn get_tracklist(&self) -> Vec<String> {
        let mut set = tokio::task::JoinSet::new();
        for (index, song) in self.songs.clone().into_iter().enumerate() {
            set.spawn(async move {
                let res = Queue::format_song(song).await;
                (index, res)
            });
        }

        let mut results = set.join_all().await;
        results.sort_unstable_by(|x, y| x.0.cmp(&y.0));
        results.iter().map(|x| x.clone().1).collect()
    }

    pub async fn current_track_over(&self) -> anyhow::Result<bool> {
        if let Some(track) = &self.current_track {
            return Ok(track.get_info().await?.playing.is_done());
        }
        Ok(false)
    }

    pub async fn play(&mut self, state: State, guild_id: Id<GuildMarker>) -> anyhow::Result<()> {
        if let Some(call_lock) = state.songbird.get(guild_id) {
            let mut call = call_lock.lock().await;
            let mut src = self.songs[self.pos].clone();

            self.stop(&mut call);
            self.set_current_track(&mut src, &mut call).await?;

            if let Some(tc) = self.text_channel {
                state
                    .http
                    .create_message(tc)
                    .content(&format!("Now playing: {}.", Queue::format_song(src).await))
                    .await?;
            }
        }
        Ok(())
    }

    pub fn stop(&self, call: &mut MutexGuard<'_, songbird::Call>) {
        call.stop();
    }

    pub fn unpause(&self) -> anyhow::Result<()> {
        if let Some(track) = &self.current_track {
            track.play()?;
        }
        Ok(())
    }

    pub fn pause(&self) -> anyhow::Result<()> {
        if let Some(track) = &self.current_track {
            track.pause()?;
        }
        Ok(())
    }

    pub async fn remove(
        &mut self,
        state: State,
        guild_id: Id<GuildMarker>,
        index: usize,
    ) -> anyhow::Result<()> {
        tracing::info!("{}:{}", index, self.pos);
        if index >= self.songs.len() {
            return Ok(());
        } else if index < self.pos {
            self.pos -= 1;
        }
        self.songs.remove(index);
        if index == self.pos {
            self.play(state, guild_id).await?;
        }
        Ok(())
    }

    pub fn insert(&mut self, song: YoutubeDl<'static>, index: usize) {
        if index < self.pos {
            self.pos += 1;
        }
        if index >= self.songs.len() {
            self.songs.push(song);
        } else {
            self.songs.insert(index, song);
        }
    }

    pub async fn push(&mut self, state: State, url: String) -> anyhow::Result<AuxMetadata> {
        let mut search = YoutubeDl::new_search(state.client.clone(), url.clone());
        let res = search.search(Some(1)).await?.collect::<Vec<_>>();
        let metadata = &res[0];
        let url = metadata.source_url.clone().unwrap();

        self.songs.push(YoutubeDl::new(state.client.clone(), url));
        Ok(metadata.clone())
    }

    pub async fn goto(
        &mut self,
        state: State,
        guild_id: Id<GuildMarker>,
        pos: usize,
    ) -> anyhow::Result<()> {
        self.pos = pos;
        self.play(state, guild_id).await?;
        Ok(())
    }
}
