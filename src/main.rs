#![feature(pattern)]
#![feature(random)]

use std::{collections::HashMap, env, future::Future, str::pattern::Pattern, sync::Arc};

use commands::{join, jump, leave, next, pause, ping, play, prev, unpause};
use music::Queue;
use parser::{Argument, ArgumentMetadata, Command, CommandWithData, TextCommand};

use reqwest::Client;
use songbird::{shards::TwilightMap, Songbird};
use tokio::sync::RwLock;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::message::EmojiReactionType,
    id::{marker::GuildMarker, Id},
};

mod commands;
mod music;
mod parser;

struct State<'a> {
    pub root_cmd: Command,
    pub http: HttpClient,
    pub songbird: Songbird,
    pub vcs: RwLock<HashMap<Id<GuildMarker>, Queue<'a>>>,
    pub client: Client,
}
impl State<'static> {
    pub fn new(
        root_cmd: Command,
        http: HttpClient,
        songbird: Songbird,
        vcs: RwLock<HashMap<Id<GuildMarker>, Queue<'static>>>,
        client: Client,
    ) -> State<'static> {
        State {
            root_cmd,
            http,
            songbird,
            vcs,
            client,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let token = &env::var("TOKEN")?;

    let root_cmd = Command::new(
        String::from("cta"),
        None,
        &[
            Command::new(
                String::from("ping"),
                Some(ping),
                &[],
                &[Argument::String(ArgumentMetadata {
                    label: String::from("text"),
                    size: 0,
                })],
            ),
            Command::new(
                String::from("jump"),
                Some(jump),
                &[],
                &[Argument::UInt(ArgumentMetadata {
                    label: String::from("amount"),
                    size: 1,
                })],
            ),
            Command::new(String::from("join"), Some(join), &[], &[]),
            Command::new(String::from("leave"), Some(leave), &[], &[]),
            Command::new(String::from("pause"), Some(pause), &[], &[]),
            Command::new(String::from("unpause"), Some(unpause), &[], &[]),
            Command::new(
                String::from("play"),
                Some(play),
                &[],
                &[Argument::String(ArgumentMetadata {
                    label: String::from("song"),
                    size: 0,
                })],
            ),
            Command::new(String::from("next"), Some(next), &[], &[]),
            Command::new(String::from("prev"), Some(prev), &[], &[]),
        ],
        &[],
    );

    let http = HttpClient::new(String::from(token));
    let user = http.current_user().await?.model().await?;

    let shard = Shard::new(
        ShardId::ONE,
        String::from(token),
        Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT
            | Intents::GUILD_MESSAGE_REACTIONS
            | Intents::GUILD_VOICE_STATES,
    );

    let shards: Vec<Shard> = vec![shard];

    let senders = TwilightMap::new(
        shards
            .iter()
            .map(|s| (s.id().number(), s.sender()))
            .collect(),
    );

    let songbird = Songbird::twilight(Arc::new(senders), user.id);

    let state = Arc::new(State::new(
        root_cmd,
        http,
        songbird,
        RwLock::new(HashMap::new()),
        Client::new(),
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
                        tracing::info!("{}", member.user.name);
                    }
                }
            }
        },
        _ => {}
    }

    Ok(())
}
