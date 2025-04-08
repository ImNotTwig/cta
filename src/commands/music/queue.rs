use std::{future::Future, pin::Pin, sync::Arc};

use songbird::input::YoutubeDl;
use twilight_model::{
    channel::message::AllowedMentions, gateway::payload::incoming::MessageCreate,
};

use crate::{music::Queue, parser::CommandWithData, State};

async fn queue_impl(s: State, m: MessageCreate, _c: CommandWithData) -> anyhow::Result<()> {
    let vcs = s.vcs.read().await.clone();
    if let Some(queue_lock) = vcs.get(&m.guild_id.unwrap()) {
        let queue = queue_lock.lock().await;
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
    let mut vcs = s.vcs.write().await.clone();
    if let Some(queue_lock) = vcs.get_mut(&m.guild_id.unwrap()) {
        let mut queue = queue_lock.lock().await;
        let mut maybe_index = None;
        if let Some(args) = c.arguments {
            if !args.is_empty() {
                maybe_index = args[0].clone().uint();
            }
        };

        let mut content = String::from("You cannot remove nothing from the queue. Nice try dummy.");
        if let Some(index) = maybe_index {
            content = if index == 0 {
                String::from("There is nothing before the queue. You cannot remove nothing.")
            } else if index as usize > queue.len() {
                String::from("Add more songs if you want to remove something from after the queue.")
            } else {
                queue
                    .remove(Arc::clone(&s), m.guild_id.unwrap(), index as usize - 1)
                    .await?;
                format!("Removed: {index} from the queue.")
            };
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
    let mut vcs = s.vcs.write().await.clone();
    if let Some(queue_lock) = vcs.get_mut(&m.guild_id.unwrap()) {
        let mut queue = queue_lock.lock().await;

        let mut url = None;
        if let Some(ref args) = c.arguments {
            if args.len() > 1 {
                url = args[1].clone().string();
            }
        };

        let mut maybe_index = None;
        if let Some(ref args) = c.arguments {
            if !args.is_empty() {
                maybe_index = args[0].clone().uint();
            }
        };

        let src = url.clone().map(|u| YoutubeDl::new(s.client.clone(), u));

        let mut content = format!(
            "You can't insert something at nothing. Perhaps you meant `insert {} {}`",
            maybe_index.unwrap_or(queue.len() as u32 - 1),
            url.clone().unwrap_or(String::from("<song here>"))
        );
        if let Some(index) = maybe_index {
            content = if index == 0 {
                String::from("There is no 0th position.")
            } else if url.is_some() {
                let s = src.unwrap();
                let song = Queue::format_song(s.clone()).await;
                queue.insert(s, index as usize - 1);
                format!("Inserted: '{song}' at {index} in the queue")
            } else {
                format!("Insert what? Insert nothing? Not possible.")
            }
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

async fn playnext_impl(s: State, m: MessageCreate, c: CommandWithData) -> anyhow::Result<()> {
    let mut vcs = s.vcs.write().await.clone();
    if let Some(queue_lock) = vcs.get_mut(&m.guild_id.unwrap()) {
        let mut queue = queue_lock.lock().await;

        let mut url = None;
        if let Some(ref args) = c.arguments {
            if !args.is_empty() {
                url = args[0].clone().string();
            }
        };

        let src = url.clone().map(|u| YoutubeDl::new(s.client.clone(), u));

        let content = if url.is_some() {
            let s = src.unwrap();
            let song = Queue::format_song(s.clone()).await;
            let pos = queue.pos();
            queue.insert(s, pos + 1);
            format!("Inserted: '{song}' at {} in the queue", pos + 2)
        } else {
            String::from("You can't play nothing.")
        };

        s.http
            .create_message(m.channel_id)
            .allowed_mentions(Some(&AllowedMentions::default()))
            .content(&content)
            .reply(m.id)
            .await?;
    }
    Ok(())
}

pub fn playnext(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    return (move |sc, mc, cc| Box::pin(playnext_impl(sc, mc, cc)))(s, m, c);
}
