use std::time::Duration;

use songbird::{
    input::{Compose, YoutubeDl},
    tracks::TrackHandle,
    Songbird,
};
use tokio::sync::MutexGuard;
use twilight_http::Client as HttpClient;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

#[derive(Clone)]
pub struct Queue<'a> {
    current_track: Option<TrackHandle>,
    current_track_len: Option<Duration>,
    songs: Vec<YoutubeDl<'a>>,
    pos: usize,
    text_channel: Option<Id<ChannelMarker>>,
}

impl Queue<'static> {
    pub fn new(
        track: Option<TrackHandle>,
        track_len: Option<Duration>,
        text_channel: Option<Id<ChannelMarker>>,
        songs: Vec<YoutubeDl<'static>>,
    ) -> Queue<'static> {
        Queue {
            current_track: track,
            current_track_len: track_len,
            songs,
            pos: 0,
            text_channel,
        }
    }

    pub async fn format_song(mut yt: YoutubeDl<'static>) -> String {
        let meta = yt.aux_metadata().await.expect("no metadata");
        return format!(
            "'{} - {}'",
            meta.artist.unwrap_or(String::from("UNKNOWN")),
            meta.title.unwrap_or(String::from("UNKNOWN")),
        );
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

    pub fn len(&self) -> usize {
        self.songs.len()
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn drop(self) {
        std::mem::drop(self);
    }

    pub async fn get_tracklist(&self) -> Vec<String> {
        let futures: Vec<_> = self
            .songs
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, yt)| {
                tokio::task::spawn(async move {
                    let res = Queue::format_song(yt).await;
                    (i, res)
                })
            })
            .collect();

        let mut results = vec![{}; futures.len()];
        for future in futures {
            let (index, res) = future.await.unwrap();
            results[index] = Some(res);
        }
        results
            .into_iter()
            .filter(|x| x.is_some())
            .map(Option::unwrap)
            .collect()
    }

    pub async fn current_track_over(&self) -> anyhow::Result<bool> {
        if let Some(track) = &self.current_track {
            return Ok(track.get_info().await?.playing.is_done());
        }
        Ok(false)
    }

    pub async fn play(
        &mut self,
        songbird: &Songbird,
        http: &HttpClient,
        guild_id: Id<GuildMarker>,
    ) -> anyhow::Result<()> {
        if let Some(call_lock) = songbird.get(guild_id) {
            let mut call = call_lock.lock().await;
            let mut src = self.songs[self.pos].clone();

            self.stop(&mut call);
            self.set_current_track(&mut src, &mut call).await?;

            if let Some(tc) = self.text_channel {
                http.create_message(tc)
                    .content(&format!("Now playing: {}.", Queue::format_song(src).await))
                    .await?;
            }
        }
        Ok(())
    }

    pub fn stop(&mut self, call: &mut MutexGuard<'_, songbird::Call>) {
        // call.stop();
    }

    pub async fn unpause(&mut self) -> anyhow::Result<()> {
        if let Some(track) = &self.current_track {
            track.play()?;
        }
        Ok(())
    }

    pub async fn pause(&mut self) -> anyhow::Result<()> {
        if let Some(track) = &self.current_track {
            track.pause()?;
        }
        Ok(())
    }

    pub async fn remove(
        &mut self,
        songbird: &Songbird,
        http: &HttpClient,
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
            self.play(songbird, http, guild_id).await?;
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

    pub fn push(&mut self, song: YoutubeDl<'static>) {
        self.songs.push(song);
    }

    pub async fn goto(
        &mut self,
        songbird: &Songbird,
        http: &HttpClient,
        guild_id: Id<GuildMarker>,
        pos: usize,
    ) -> anyhow::Result<()> {
        self.pos = pos;
        self.play(songbird, http, guild_id).await?;
        Ok(())
    }
}
