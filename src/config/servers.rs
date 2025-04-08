use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use twilight_model::id::{
    marker::{ChannelMarker, EmojiMarker, MessageMarker, RoleMarker, UserMarker},
    Id,
};

type OptionId<T> = Option<Id<T>>;
type OptionMap<K, V> = Option<HashMap<K, V>>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub begin: Duration,
    pub end: Duration,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChannelSet {
    pub log: OptionId<ChannelMarker>,
    pub spam: OptionId<ChannelMarker>,

    pub significant_reactions: OptionMap<Id<EmojiMarker>, Id<ChannelMarker>>,
}

/// `ServerConfig` represents the configuration for any given Discord guild, and contains many settings
/// which an admin may configure.
/// NOTE: Any value that is None disables related behaviors
/// i.e: if `significant_reaction_count` is None then this bot will never post significant reactions,
/// The same applies if `channels.significant_reactions` is None.
#[derive(Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    prefix: Option<String>,

    pub channels: ChannelSet,

    pub reaction_roles: OptionMap<(Id<MessageMarker>, Id<EmojiMarker>), Id<RoleMarker>>,

    pub reminders: OptionMap<Id<UserMarker>, Vec<Reminder>>,

    pub mute_role: OptionId<RoleMarker>,
    pub blacklisted_words: Option<Box<[String]>>,

    pub auto_responses: OptionMap<String, String>,
    pub auto_reacts: OptionMap<String, Id<EmojiMarker>>,

    pub significant_reaction_count: Option<u32>,
}

impl ServerConfig {
    pub const fn new() -> Self {
        Self {
            prefix: None,
            channels: ChannelSet {
                log: None,
                spam: None,
                significant_reactions: None,
            },
            reaction_roles: None,
            reminders: None,
            mute_role: None,
            blacklisted_words: None,
            auto_responses: None,
            auto_reacts: None,
            significant_reaction_count: None,
        }
    }

    pub fn prefix(&self) -> String {
        self.prefix.clone().unwrap_or_else(|| String::from("~"))
    }

    pub fn set_prefix(&mut self, pfx: &str) {
        self.prefix = Some(String::from(pfx));
    }
}
