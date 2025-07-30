use std::{collections::HashMap, str::pattern::Pattern, sync::Arc};

use reqwest::Client;
use songbird::Songbird;
use tokio::sync::Mutex;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Event;
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::message::EmojiReactionType,
    id::{marker::GuildMarker, Id},
};

use crate::{
    config::servers::ServerConfig,
    music::Queue,
    parser::{Command, CommandWithData, TextCommand},
};

pub trait Handler {
    async fn handle_event(self, event: Event) -> anyhow::Result<()>;

    async fn leave_empty_vcs(self) -> anyhow::Result<()>;
    async fn check_done_vcs(self) -> anyhow::Result<()>;

    async fn generate_configs(self) -> anyhow::Result<()>;
    async fn write_configs_to_file(&self) -> anyhow::Result<()>;
    async fn read_configs_from_file(&self) -> anyhow::Result<()>;
}

pub struct StateRef<'a> {
    pub root_cmd: Command,
    pub http: HttpClient,
    pub songbird: Songbird,
    pub vcs: Mutex<HashMap<Id<GuildMarker>, Arc<Mutex<Queue<'a>>>>>,
    pub server_configs: Mutex<HashMap<Id<GuildMarker>, ServerConfig>>,
    pub client: Client,
    pub cache: InMemoryCache,
}

pub type State = Arc<StateRef<'static>>;

async fn get_empty_vcs(state: State) -> Vec<Id<GuildMarker>> {
    let mut guilds = vec![];
    for i in state.vcs.lock().await.clone().iter() {
        if let Some(call_lock) = state.songbird.get(*i.0) {
            if let Some(vc) = call_lock.lock().await.current_channel() {
                let member_count = state
                    .cache
                    .voice_channel_states(vc.0.into())
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
                    guilds.push(*i.0);
                }
            }
        }
    }
    guilds
}

impl Handler for State {
    async fn generate_configs(self) -> anyhow::Result<()> {
        let guilds = self.http.current_user_guilds().await?.model().await?;
        let configs = self.server_configs.lock().await.clone();
        for guild in guilds {
            if configs.get(&guild.id).is_none() {
                _ = self
                    .server_configs
                    .lock()
                    .await
                    .insert(guild.id, ServerConfig::new());
            }
        }
        Ok(())
    }

    async fn write_configs_to_file(&self) -> anyhow::Result<()> {
        let configs = self.server_configs.lock().await.clone();
        for (guild, config) in configs.iter() {
            let data = bincode::serde::encode_to_vec(config, bincode::config::standard())?;
            println!("{} : {:?}", guild, String::from_utf8(data)?);
        }
        Ok(())
    }

    async fn read_configs_from_file(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn leave_empty_vcs(self) -> anyhow::Result<()> {
        loop {
            let guilds = get_empty_vcs(Arc::clone(&self)).await;
            for guild in guilds {
                tokio::spawn((async move |guild, state| {
                    tokio::time::sleep(std::time::Duration::SECOND * 60).await;
                    let guilds = get_empty_vcs(Arc::clone(&state)).await;
                    let this_guild: Vec<_> = guilds.iter().filter(|x| **x == guild).collect();

                    if !this_guild.is_empty() {
                        state.songbird.leave(guild).await.unwrap();
                        state.songbird.remove(guild).await.unwrap();
                        state.vcs.lock().await.remove(&guild).unwrap();
                    }
                })(guild.clone(), Arc::clone(&self)))
                .await?;
            }
        }
    }

    async fn check_done_vcs(self) -> anyhow::Result<()> {
        loop {
            let mut guilds = vec![];
            let queues = self.vcs.lock().await.clone();
            for i in queues.iter() {
                let vc = i.1.lock().await;
                if let Ok(over) = vc.current_track_over().await {
                    if over && vc.len() - 1 == vc.pos() {
                        guilds.push(*i.0);
                    }
                } else {
                    guilds.push(*i.0);
                }
            }
            for i in guilds {
                let mut queue = queues.get(&i).unwrap().lock().await;
                let pos = queue.pos();
                if pos < queue.len() - 1 {
                    queue.goto(Arc::clone(&self), i, pos + 1).await?;
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
                if txt_cmd.clone().collect::<Vec<String>>().is_empty() {
                    return Ok(());
                }
                let configs = self.server_configs.lock().await.clone();
                let pfx = configs.get(&msg.guild_id.unwrap()).unwrap().prefix();

                if pfx.is_prefix_of(txt_cmd.first()) {
                    if let Some(prefix_commmand) = txt_cmd.first().strip_prefix(&pfx) {
                        if let Some(subcommand) = self.root_cmd.find_command(prefix_commmand) {
                            if let Some(func) = subcommand.function {
                                _ = txt_cmd.next();
                                let command_with_data = CommandWithData::new(txt_cmd, *subcommand)?;
                                _ = tokio::spawn(async move {
                                    (func)(Arc::clone(&self), *msg.clone(), command_with_data)
                                        .await
                                        .unwrap();
                                });
                            }
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
    pub const fn new(
        root_cmd: Command,
        http: HttpClient,
        songbird: Songbird,
        vcs: Mutex<HashMap<Id<GuildMarker>, Arc<Mutex<Queue<'static>>>>>,
        server_configs: Mutex<HashMap<Id<GuildMarker>, ServerConfig>>,
        client: Client,
        cache: InMemoryCache,
    ) -> Self {
        StateRef {
            root_cmd,
            http,
            songbird,
            vcs,
            server_configs,
            client,
            cache,
        }
    }
}
