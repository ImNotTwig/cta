use std::{future::Future, net::SocketAddr, pin::Pin};

use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Request};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client as HyperClient},
    rt::TokioExecutor,
};
use twilight_lavalink::{
    http::{LoadResultData, LoadedTracks},
    model::{outgoing, Track},
};
use twilight_model::{
    channel::message::AllowedMentions,
    gateway::payload::{incoming::MessageCreate, outgoing::UpdateVoiceState},
};

use crate::{parser::CommandWithData, State};

async fn join_impl(s: State, m: MessageCreate, c: CommandWithData) -> anyhow::Result<()> {
    let vc = s
        .http
        .user_voice_state(m.guild_id.unwrap().clone(), m.author.id.clone())
        .await?
        .model()
        .await?
        .channel_id
        .unwrap();

    s.sender.command(&UpdateVoiceState::new(
        m.guild_id.unwrap().clone(),
        vc,
        true,
        false,
    ))?;

    let player = s.lavalink.player(m.guild_id.unwrap()).await?;
    println!("{}", player.channel_id().unwrap());

    let (parts, body) = twilight_lavalink::http::load_track(
        player.node().config().address,
        // "https://www.youtube.com/watch?v=9FLRHejWAo8",
        "https://soundcloud.com/twig-559499225/shit-got-weird/s-zbgxqroVXTZ?si=dd85813872a54f67ab56178ba0213216",
        &player.node().config().authorization,
    )?
    .into_parts();

    println!("{}", String::from_utf8(body.to_ascii_lowercase()).unwrap());

    let hyper: HyperClient<HttpConnector, Full<Bytes>> =
        HyperClient::builder(TokioExecutor::new()).build_http();

    let req = Request::from_parts(parts, Full::from(body));
    let res = hyper.request(req).await?;
    let response_bytes = res.collect().await?.to_bytes();
    let loaded = serde_json::from_slice::<LoadedTracks>(&response_bytes)?;

    match loaded.data {
        LoadResultData::Track(track) => {
            player.send(outgoing::Play::from((m.guild_id.unwrap(), &track.encoded)))?;
        }
        LoadResultData::Search(track) => {
            player.send(outgoing::Play::from((
                m.guild_id.unwrap(),
                &track.first().unwrap().encoded,
            )))?;
        }
        _ => {}
    };

    s.http
        .create_message(m.channel_id)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .content(&format!("Joined <#{}>!", vc))
        .reply(m.id)
        .await?;

    Ok(())
}
pub fn join(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    Box::pin(join_impl(s, m, c))
}

async fn leave_impl(s: State, m: MessageCreate, c: CommandWithData) -> anyhow::Result<()> {
    s.sender.command(&UpdateVoiceState::new(
        m.guild_id.unwrap().clone(),
        None,
        true,
        false,
    ))?;

    s.http
        .create_message(m.channel_id)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .content(&format!("I've Left and cleared the queue!"))
        .reply(m.id)
        .await?;

    Ok(())
}
pub fn leave(
    s: State,
    m: MessageCreate,
    c: CommandWithData,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
    Box::pin(leave_impl(s, m, c))
}
