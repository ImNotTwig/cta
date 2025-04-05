#![feature(pattern)]
#![feature(random)]
#![feature(duration_constants)]

use core::time;
use std::{collections::HashMap, env, future::Future, str::pattern::Pattern, sync::Arc};

use music::Queue;
use parser::{Argument, ArgumentMetadata, Command, CommandWithData, TextCommand};

use reqwest::Client;
use songbird::{shards::TwilightMap, Songbird};
use tokio::sync::RwLock;
use twilight_cache_inmemory::{CacheableCurrentUser, DefaultInMemoryCache, InMemoryCache};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::message::EmojiReactionType,
    id::{marker::GuildMarker, Id},
};

mod commands;
mod config;
mod music;
mod parser;

struct State<'a> {
    pub root_cmd: Command,
    pub http: HttpClient,
    pub songbird: Songbird,
    pub vcs: RwLock<HashMap<Id<GuildMarker>, Queue<'a>>>,
    pub client: Client,
    pub cache: InMemoryCache,
}
impl State<'static> {
    pub fn new(
        root_cmd: Command,
        http: HttpClient,
        songbird: Songbird,
        vcs: RwLock<HashMap<Id<GuildMarker>, Queue<'static>>>,
        client: Client,
        cache: InMemoryCache,
    ) -> State<'static> {
        State {
            root_cmd,
            http,
            songbird,
            vcs,
            client,
            cache,
        }
    }
}

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

    let state = Arc::new(State::new(
        commands::rootcmd(),
        http,
        songbird,
        RwLock::new(HashMap::new()),
        Client::new(),
        cache,
    ));

    tracing::info!("Logged in as: {}", user.name);

    let mut set = tokio::task::JoinSet::new();
    for s in shards {
        set.spawn(tokio::spawn(runner(s, Arc::clone(&state))));
        set.spawn(tokio::spawn(check_done_vcs(Arc::clone(&state))));
    }

    set.join_next().await;

    Ok(())
}

async fn leave_empty_vcs(state: Arc<State<'static>>) {
    loop {
        let mut guilds = vec![];
        for i in state.vcs.read().await.iter() {
            let vc = state
                .songbird
                .get(*i.0)
                .unwrap()
                .lock()
                .await
                .current_channel()
                .unwrap()
                .0
                .into();

            let member_count = state
                .cache
                .voice_channel_states(vc)
                .map(|voice_states| {
                    let mut users = voice_states
                        .map(|v| state.cache.user(v.user_id()))
                        .collect::<Option<Vec<_>>>()
                        .unwrap();
                    users.retain(|u| !u.bot);
                    users.len()
                })
                .unwrap();

            if member_count == 0 {
                guilds.push(i.0.clone());
            }
        }
        for guild in guilds {
            state.songbird.leave(guild).await.unwrap();
            let mut vcs = state.vcs.write().await;
            if let Some(vc) = vcs.remove(&guild) {
                vc.drop();
            }
        }
    }
}

async fn check_done_vcs(state: Arc<State<'static>>) {
    loop {
        let mut guilds = vec![];
        for i in state.vcs.read().await.iter() {
            if let Ok(over) = i.1.current_track_over().await {
                if over {
                    guilds.push(i.0.clone());
                }
            } else {
                guilds.push(i.0.clone());
            }
        }
        for i in guilds {
            state
                .vcs
                .write()
                .await
                .get_mut(&i)
                .unwrap()
                .next(&state.songbird, &state.http, i)
                .await
                .unwrap();
        }
    }
}

async fn runner(mut shard: Shard, state: Arc<State<'static>>) {
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
                let s = Arc::clone(&state);
                async move { handle_event(s, event).await }
            });
        };
        tokio::spawn(leave_empty_vcs(Arc::clone(&state)));
    }
}

fn spawn(fut: impl Future<Output = anyhow::Result<()>> + Send + 'static) {
    tokio::spawn(async move {
        if let Err(why) = fut.await {
            tracing::debug!("handler error: {:?}", why);
        }
    });
}

async fn handle_event(state: Arc<State<'static>>, event: Event) -> anyhow::Result<()> {
    state.songbird.process(&event).await;
    state.cache.update(&event);

    match event {
        //TODO: unhardcode the prefix
        Event::MessageCreate(msg) => {
            let mut txt_cmd = TextCommand::new(&msg.content);
            if txt_cmd
                .clone()
                .into_iter()
                .collect::<Vec<String>>()
                .is_empty()
            {
                return Ok(());
            };
            if "".is_prefix_of(txt_cmd.first()) {
                if let Some(subcommand) = state.root_cmd.clone().find_command(
                    txt_cmd
                        .clone()
                        .first()
                        .strip_prefix("~")
                        .unwrap_or(txt_cmd.first()),
                ) {
                    if let Some(func) = subcommand.function {
                        _ = txt_cmd.next();
                        let command_with_data = CommandWithData::new(txt_cmd, *subcommand);
                        spawn((func)(Arc::clone(&state), *msg.clone(), command_with_data));
                    }
                }
            }
        }
        Event::ReactionAdd(reaction) => match &reaction.emoji {
            EmojiReactionType::Custom {
                animated: _,
                id: _,
                name: _,
            } => {
                // println!("{}", name.clone().unwrap())
            }

            EmojiReactionType::Unicode { name } => {
                if name == "⭐" {
                    if let Some(member) = &reaction.member {
                        // tracing::info!("{}", member.user.name);
                    }
                }
            }
        },
        _ => {}
    }

    Ok(())
}
