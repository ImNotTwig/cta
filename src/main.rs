#![feature(pattern)]

use std::sync::Arc;

use commands::ping;
use parser::get_message_command;
use parser::Command;

use tracing_subscriber;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::EmojiReactionType;

mod commands;
mod parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let token = "";

    let rc = Command::new(
        String::from("cta"),
        None,
        &[Command {
            name: String::from("ping"),
            function: Some(ping),
            subcommands: None,
            arguments: Some(Box::new([parser::Argument::String(String::from("text"))])),
        }],
        &[],
    );

    let mut shard = Shard::new(
        ShardId::ONE,
        String::from(token),
        Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT | Intents::GUILD_MESSAGE_REACTIONS,
    );

    let http = Arc::new(HttpClient::new(String::from(token)));
    tracing::info!(
        "Logged in as: {}",
        http.current_user().await?.model().await?.name
    );

    loop {
        let item = shard
            .next_event(EventTypeFlags::REACTION_ADD | EventTypeFlags::MESSAGE_CREATE)
            .await
            .unwrap();
        let event = match item {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!("error receiving event: {}", err);
                continue;
            }
        };

        tokio::spawn(handle_event(rc.clone(), event, Arc::clone(&http)));
    }
}

async fn handle_event(
    root_command: Command,
    event: Event,
    http: Arc<HttpClient>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {
            if let Some(txt_cmd) = get_message_command(&msg.content, "~") {
                if let Some(subcommand) = root_command.find_command(txt_cmd) {
                    if let Some(func) = subcommand.function {
                        (func)(http.clone(), *msg.clone(), *subcommand).await?;
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
