#![feature(pattern)]

use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

use reqwest::Client;
use state::{Handler, State, StateRef};
use tokio::sync::Mutex;
use twilight_cache_inmemory::DefaultInMemoryCache;
use twilight_gateway::{EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client as HttpClient;
use twilight_lavalink::Lavalink;

mod commands;
mod config;
mod parser;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();

    let token = &env::var("TOKEN")?;

    let cache = DefaultInMemoryCache::builder()
        .message_cache_size(10)
        .build();

    let http = HttpClient::new(String::from(token));
    let user = http.current_user().await?.model().await?;

    let lavalink_auth = "69420";
    let lavalink_host: SocketAddr = "0.0.0.0:2333".parse().unwrap();
    let lavalink = Lavalink::new(user.id, 1);
    lavalink.add(lavalink_host, lavalink_auth).await?;

    let shard = Shard::new(ShardId::ONE, String::from(token), Intents::all());

    let s = Arc::new(StateRef::new(
        commands::rootcmd(),
        http,
        Mutex::new(HashMap::new()),
        Client::new(),
        cache,
        shard.sender().clone(),
        lavalink,
    ));

    Arc::clone(&s).generate_configs().await?;
    tracing::info!("Logged in as: {}", user.name);

    let mut set = tokio::task::JoinSet::new();
    set.spawn(tokio::spawn(runner(shard, Arc::clone(&s))));

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
