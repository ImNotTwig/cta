use anyhow::Error;
use lavalink_rs::{
    client::LavalinkClient, model::events, node::NodeBuilder, prelude::NodeDistributionStrategy,
};
use poise::serenity_prelude::{self as serenity};
use songbird::SerenityInit;

use crate::commands::{insert, join, leave, nowplaying, pause, play, playnext, queue, skip};

pub mod commands;
pub mod hooks;
pub mod util;

pub struct Data {
    pub lavalink: LavalinkClient,
}

pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let framework = poise::Framework::<Data, Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                join(),
                play(),
                leave(),
                skip(),
                pause(),
                nowplaying(),
                insert(),
                playnext(),
                queue(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(String::from("~")),
                ..Default::default()
            },

            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let mut events = events::Events::default();
                events.track_start = Some(hooks::track_start);
                let node_local = NodeBuilder {
                    hostname: String::from("0.0.0.0:2333"),
                    is_ssl: false,
                    events: events::Events::default(),
                    password: String::from("69420"),
                    user_id: ctx.cache.current_user().id.into(),
                    session_id: None,
                };

                let lavalink = LavalinkClient::new(
                    events,
                    vec![node_local],
                    NodeDistributionStrategy::round_robin(),
                )
                .await;

                Ok(Data { lavalink })
            })
        })
        .build();

    start(framework).await?;

    Ok(())
}

pub async fn start(framework: poise::Framework<Data, Error>) -> anyhow::Result<()> {
    let mut client = serenity::ClientBuilder::new(
        std::env::var("TOKEN").expect("Set $TOKEN to your Discord token."),
        serenity::GatewayIntents::all(),
    )
    .register_songbird()
    .framework(framework)
    .await?;
    client.start().await?;
    Ok(())
}
