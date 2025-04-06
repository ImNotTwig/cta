use std::{collections::HashMap, str::pattern::Pattern, sync::Arc};

use reqwest::Client;
use songbird::Songbird;
use tokio::sync::RwLock;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Event;
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::message::EmojiReactionType,
    id::{marker::GuildMarker, Id},
};

use crate::{
    music::Queue,
    parser::{Command, CommandWithData, TextCommand},
};

pub trait Handler {
    async fn handle_event(self, event: Event) -> anyhow::Result<()>;
    async fn leave_empty_vcs(self) -> anyhow::Result<()>;
    async fn check_done_vcs(self) -> anyhow::Result<()>;
}

pub struct StateRef<'a> {
    pub root_cmd: Command,
    pub http: HttpClient,
    pub songbird: Songbird,
    pub vcs: RwLock<HashMap<Id<GuildMarker>, Queue<'a>>>,
    pub client: Client,
    pub cache: InMemoryCache,
}

pub type State = Arc<StateRef<'static>>;

impl Handler for State {
    async fn leave_empty_vcs(self) -> anyhow::Result<()> {
        loop {
            let mut guilds = vec![];
            for i in self.vcs.read().await.iter() {
                if let Some(call_lock) = self.songbird.get(*i.0) {
                    let call = call_lock.lock().await;

                    let vc = call.current_channel().expect("Not in a channel.").0;

                    let member_count = self
                        .cache
                        .voice_channel_states(vc.into())
                        .map(|voice_states| {
                            let mut users = voice_states
                                .map(|v| self.cache.user(v.user_id()))
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
            }
            for guild in guilds {
                self.songbird.leave(guild).await.unwrap();
                let mut vcs = self.vcs.write().await;
                if let Some(vc) = vcs.remove(&guild) {
                    vc.drop();
                }
            }
        }
    }

    async fn check_done_vcs(self) -> anyhow::Result<()> {
        loop {
            let mut guilds = vec![];
            for i in self.vcs.read().await.iter() {
                if let Ok(over) = i.1.current_track_over().await {
                    if over {
                        guilds.push(i.0.clone());
                    }
                } else {
                    guilds.push(i.0.clone());
                }
            }
            for i in guilds {
                let mut queues = self.vcs.write().await;
                let queue = queues.get_mut(&i).unwrap();
                if queue.pos() < queue.len() - 1 {
                    queue
                        .goto(&self.songbird, &self.http, i, queue.pos() + 1)
                        .await?;
                }
            }
        }
    }

    async fn handle_event(self, event: Event) -> anyhow::Result<()> {
        self.songbird.process(&event).await;
        self.cache.update(&event);

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
                    if let Some(subcommand) = self.root_cmd.clone().find_command(
                        txt_cmd
                            .clone()
                            .first()
                            .strip_prefix("~")
                            .unwrap_or(txt_cmd.first()),
                    ) {
                        if let Some(func) = subcommand.function {
                            _ = txt_cmd.next();
                            let command_with_data = CommandWithData::new(txt_cmd, *subcommand);
                            _ = tokio::spawn(async move {
                                (func)(Arc::clone(&self), *msg.clone(), command_with_data)
                                    .await
                                    .unwrap();
                            });
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
}

impl StateRef<'static> {
    pub fn new(
        root_cmd: Command,
        http: HttpClient,
        songbird: Songbird,
        vcs: RwLock<HashMap<Id<GuildMarker>, Queue<'static>>>,
        client: Client,
        cache: InMemoryCache,
    ) -> StateRef<'static> {
        StateRef {
            root_cmd,
            http,
            songbird,
            vcs,
            client,
            cache,
        }
    }
}
