use std::time::Duration;

use songbird::{
    input::{Compose, YoutubeDl},
    tracks::TrackHandle,
    Songbird,
};
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

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn drop(self) {
        std::mem::drop(self);
    }

    pub async fn get_tracklist(&self) -> Vec<String> {
        async fn run(mut yt: YoutubeDl<'static>) -> String {
            let meta = yt.aux_metadata().await.expect("no metadata");
            return format!(
                "'{} - {}'",
                meta.title.unwrap_or(String::from("UNKNOWN")),
                meta.artist.unwrap_or(String::from("UNKNOWN"))
            );
        }

        let futs = self.songs.clone().into_iter().map(|x| run(x));
        futures::future::join_all(futs).await
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
            self.current_track = Some(call.play_input(self.songs[self.pos].clone().into()));

            let mut src = self.songs[self.pos].clone();

            if let Ok(meta) = src.aux_metadata().await {
                self.current_track_len = if let Ok(meta) = src.aux_metadata().await {
                    meta.duration
                } else {
                    None
                };

                if let Some(tc) = self.text_channel {
                    let content = format!(
                        "Now playing: {} - {}.",
                        meta.title.unwrap(),
                        meta.artist.unwrap()
                    );

                    http.create_message(tc).content(&content).await?;
                }
            }
        }
        Ok(())
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

    pub fn push(&mut self, song: YoutubeDl<'static>) {
        self.songs.push(song);
    }

    pub async fn prev(
        &mut self,
        songbird: &Songbird,
        http: &HttpClient,
        guild_id: Id<GuildMarker>,
    ) -> anyhow::Result<()> {
        if self.pos as i32 - 1 < 0 {
            return Ok(());
        }
        self.pos -= 1;

        let mut src = self.songs[self.pos].clone();

        if let Some(call_lock) = songbird.get(guild_id) {
            let mut call = call_lock.lock().await;
            let handle = call.play_input(src.clone().into());

            if let Ok(meta) = src.aux_metadata().await {
                if let Some(tc) = self.text_channel {
                    let content = format!(
                        "Now playing: {} - {}.",
                        meta.track.unwrap_or(String::from("UNKNOWN")),
                        meta.artist.unwrap_or(String::from("UNKNOWN")),
                    );

                    http.create_message(tc).content(&content).await?;
                }

                if let Some(track) = self.current_track.as_mut() {
                    let _ = track.stop();
                }
                self.current_track = Some(handle);

                self.current_track_len = if let Ok(meta) = src.aux_metadata().await {
                    meta.duration
                } else {
                    None
                };
            }
        }

        Ok(())
    }

    pub async fn next(
        &mut self,
        songbird: &Songbird,
        http: &HttpClient,
        guild_id: Id<GuildMarker>,
    ) -> anyhow::Result<()> {
        if self.pos + 1 >= self.songs.len() {
            return Ok(());
        }
        self.pos += 1;

        let mut src = self.songs[self.pos].clone();

        if let Some(call_lock) = songbird.get(guild_id) {
            let mut call = call_lock.lock().await;
            let handle = call.play_input(src.clone().into());

            if let Ok(meta) = src.aux_metadata().await {
                if let Some(tc) = self.text_channel {
                    let content = format!(
                        "Now playing: {} - {}.",
                        meta.title.unwrap(),
                        meta.artist.unwrap()
                    );

                    http.create_message(tc).content(&content).await?;
                }

                if let Some(track) = self.current_track.as_mut() {
                    let _ = track.stop();
                }
                self.current_track = Some(handle);

                self.current_track_len = if let Ok(meta) = src.aux_metadata().await {
                    meta.duration
                } else {
                    None
                };
            }
        }

        Ok(())
    }
}
