use std::{future::Future, pin::Pin};

use songbird::input::{Compose, YoutubeDl};
use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::{music::Queue, parser::CommandWithData, State};

async fn queue_impl(s: State, m: MessageCreate, _c: CommandWithData) -> anyhow::Result<()> {
    if let Some(queue) = s.vcs.read().await.get(&m.guild_id.unwrap()) {
        let content = queue.get_tracklist().await;

        let mut str = String::from("Queue\n```\n");
        let mut i = 0;
        for j in content.iter() {
            if queue.pos() == i {
                str += &(String::from("-> ") + j + "\n");
            } else {
                str += &(format!("{}: ", i + 1) + j + "\n");
            }
            i += 1;
        }
        str += "```";

        s.http
            .create_message(m.channel_id)
            .allowed_mentions(Some(&AllowedMentions::default()))
            .content(&format!("{str}"))
            .reply(m.id)
            .await?;
    } else {
        return Ok(());
    };

    Ok(())
}
pub fn queue(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(queue_impl(sc, mc, cc)))(s, m, c);
}

async fn remove_impl(s: State, m: MessageCreate, c: CommandWithData) -> anyhow::Result<()> {
    let mut maybe_index = None;
    if let Some(queue) = s.vcs.write().await.get_mut(&m.guild_id.unwrap()) {
        if let Some(args) = c.arguments {
            if args.len() > 0 {
                maybe_index = Some(args[0].clone().uint());
            }
        };

        let (content, valid) = if let Some(index) = maybe_index {
            if index < 1 {
                (
                    String::from("There is nothing before the queue. You cannot remove nothing."),
                    false,
                )
            } else if index as usize > queue.len() {
                (
                    String::from(
                        "Add more songs if you want to remove something from after the queue.",
                    ),
                    false,
                )
            } else {
                (format!("Removed: {index} from the queue."), true)
            }
        } else {
            (
                String::from("You cannot remove nothing from the queue. Nice try dummy."),
                false,
            )
        };

        if valid {
            queue
                .remove(
                    &s.songbird,
                    &s.http,
                    m.guild_id.unwrap(),
                    maybe_index.expect("Impossible") as usize - 1,
                )
                .await?;
        }

        s.http
            .create_message(m.channel_id)
            .allowed_mentions(Some(&AllowedMentions::default()))
            .content(&content)
            .reply(m.id)
            .await?;
    }
    Ok(())
}

pub fn remove(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(remove_impl(sc, mc, cc)))(s, m, c);
}

async fn insert_impl(s: State, m: MessageCreate, c: CommandWithData) -> anyhow::Result<()> {
    if let Some(queue) = s.vcs.write().await.get_mut(&m.guild_id.unwrap()) {
        let mut url = String::new();
        if let Some(ref args) = c.arguments {
            if args.len() > 1 {
                url = args[1].clone().string();
            }
        };

        let mut maybe_index = None;
        if let Some(ref args) = c.arguments {
            if args.len() > 0 {
                maybe_index = Some(args[0].clone().uint());
            }
        };

        let mut src = YoutubeDl::new(s.client.clone(), url.clone());

        let (content, valid) = if let Some(index) = maybe_index {
            if index < 1 {
                (String::from("There is no 0th position."), false)
            } else if url == "" {
                (String::from("Insert what? Nothing? Not possible."), false)
            } else {
                let song = Queue::format_song(src.clone()).await;
                (format!("Inserted: '{song}' at {index} in the queue"), true)
            }
        } else {
            (
                String::from("You can't insert something at nothing."),
                false,
            )
        };

        if valid {
            queue.insert(src, maybe_index.expect("Impossible") as usize - 1);
        }

        s.http
            .create_message(m.channel_id)
            .allowed_mentions(Some(&AllowedMentions::default()))
            .content(&content)
            .reply(m.id)
            .await?;
    }
    Ok(())
}

pub fn insert(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(insert_impl(sc, mc, cc)))(s, m, c);
}
