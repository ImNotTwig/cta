use std::{collections::HashMap, str::pattern::Pattern, sync::Arc};

use reqwest::Client;
use tokio::sync::Mutex;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{Event, MessageSender};
use twilight_http::Client as HttpClient;
use twilight_lavalink::Lavalink;
use twilight_model::{
    channel::message::EmojiReactionType,
    id::{marker::GuildMarker, Id},
};

use crate::{
    config::servers::ServerConfig,
    parser::{Command, CommandWithData, TextCommand},
};

pub trait Handler {
    async fn handle_event(self, event: Event) -> anyhow::Result<()>;

    async fn generate_configs(self) -> anyhow::Result<()>;
    async fn write_configs_to_file(&self) -> anyhow::Result<()>;
    async fn read_configs_from_file(&self) -> anyhow::Result<()>;
}

pub struct StateRef {
    pub root_cmd: Command,
    pub http: HttpClient,
    pub server_configs: Mutex<HashMap<Id<GuildMarker>, ServerConfig>>,
    pub client: Client,
    pub cache: InMemoryCache,
    pub sender: MessageSender,
    pub lavalink: Lavalink,
}

pub type State = Arc<StateRef>;

impl Handler for State {
    async fn generate_configs(self) -> anyhow::Result<()> {
        let guilds = self.http.current_user_guilds().await?.model().await?;
        let configs = self.server_configs.lock().await.clone();
        for guild in guilds {
            if !configs.contains_key(&guild.id) {
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
        for (guild, config) in &configs {
            let data = bincode::serde::encode_to_vec(config, bincode::config::standard())?;
            println!("{} : {:?}", guild, String::from_utf8(data)?);
        }
        Ok(())
    }

    async fn read_configs_from_file(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn handle_event(self, event: Event) -> anyhow::Result<()> {
        self.lavalink.process(&event).await?;
        self.cache.update(&event);

        match event {
            //TODO: unhardcode the prefix
            Event::MessageCreate(msg) => {
                let mut txt_cmd = TextCommand::new(&msg.content);
                if txt_cmd.clone().next().is_none() {
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
                                    let res =
                                        (func)(Arc::clone(&self), *msg.clone(), command_with_data)
                                            .await;
                                    if res.is_err() {
                                        _ = self.http
                                            .create_message(msg.channel_id)
                                            .content("Can't play whatever the fuck you just tried to add.")
                                            .await;
                                    }
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

impl StateRef {
    pub const fn new(
        root_cmd: Command,
        http: HttpClient,
        server_configs: Mutex<HashMap<Id<GuildMarker>, ServerConfig>>,
        client: Client,
        cache: InMemoryCache,
        sender: MessageSender,
        lavalink: Lavalink,
    ) -> Self {
        Self {
            root_cmd,
            http,
            server_configs,
            client,
            cache,
            sender,
            lavalink,
        }
    }
}
