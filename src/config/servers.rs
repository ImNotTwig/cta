use std::{collections::HashMap, time::Duration};

use bincode::{config, Decode, Encode};
use twilight_model::id::{
    marker::{ChannelMarker, EmojiMarker, MessageMarker, RoleMarker, UserMarker},
    Id,
};

type OptionId<T> = Option<Id<T>>;
type OptionMap<K, V> = Option<HashMap<K, V>>;

pub struct Reminder {
    pub begin: Duration,
    pub end: Duration,
    pub message: String,
}

pub struct ChannelSet {
    pub log: OptionId<ChannelMarker>,
    pub spam: OptionId<ChannelMarker>,

    pub significant_reactions: OptionMap<Id<EmojiMarker>, Id<ChannelMarker>>,
}

/// ServerConfig represents the configuration for any given Discord guild, and contains many settings
/// which an admin may configure.
/// NOTE: Any value that is None disables related behaviors
/// i.e: if `significant_reaction_count` is None then this bot will never post significant reactions,
/// The same applies if `significant_reaction_channel` is None.
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
    pub fn prefix(&self) -> String {
        self.prefix.clone().unwrap_or(String::from("~"))
    }
}
