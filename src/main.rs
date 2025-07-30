#![feature(pattern)]
#![feature(random)]
#![feature(duration_constants)]

use std::{collections::HashMap, env, sync::Arc};

use reqwest::Client;
use songbird::{shards::TwilightMap, Songbird};
use state::{Handler, State, StateRef};
use tokio::sync::Mutex;
use twilight_cache_inmemory::DefaultInMemoryCache;
use twilight_gateway::{EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client as HttpClient;

mod commands;
mod config;
mod music;
mod parser;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let token = &env::var("TOKEN")?;

    let cache = DefaultInMemoryCache::builder()
        .message_cache_size(10)
        .build();

    let http = HttpClient::new(String::from(token));
    let user = http.current_user().await?.model().await?;

    let shard = Shard::new(ShardId::ONE, String::from(token), Intents::all());

    let shards: Vec<Shard> = vec![shard];

    let senders = TwilightMap::new(
        shards
            .iter()
            .map(|s| (s.id().number(), s.sender()))
            .collect(),
    );

    let songbird = Songbird::twilight(Arc::new(senders), user.id);

    let s = Arc::new(StateRef::new(
        commands::rootcmd(),
        http,
        songbird,
        Mutex::new(HashMap::new()),
        Mutex::new(HashMap::new()),
        Client::new(),
        cache,
    ));
    Arc::clone(&s).generate_configs().await?;
    tracing::info!("Logged in as: {}", user.name);

    let mut set = tokio::task::JoinSet::new();
    for shard in shards {
        set.spawn(tokio::spawn(runner(shard, Arc::clone(&s))));
        set.spawn(tokio::spawn(Arc::clone(&s).check_done_vcs()));
        set.spawn(tokio::spawn(Arc::clone(&s).leave_empty_vcs()));
    }

    set.join_next().await;

    Ok(())
}

async fn runner(mut shard: Shard, s: State) -> anyhow::Result<()> {
    loop {
        if let Some(item) = shard.next_event(EventTypeFlags::all()).await {
            let event = match item {
                Ok(e) => e,
                Err(err) => {
                    tracing::warn!("error receiving event: {}", err);
                    continue;
                }
            };
            tokio::spawn({
                let s = Arc::clone(&s);
                async move {
                    match s.handle_event(event).await {
                        Ok(()) => {}
                        Err(why) => {
                            tracing::debug!("Error processing event: {why}.");
                        }
                    }
                }
            });
        }
    }
}
