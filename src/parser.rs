use std::{future::Future, pin::Pin, sync::Arc};

use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::incoming::MessageCreate;

struct TextCommand<'a> {
    words: Box<[&'a str]>,
    ptr: usize,
}
impl<'a> TextCommand<'a> {
    pub fn new(message: &str) -> TextCommand {
        TextCommand {
            words: message.split_whitespace().collect(),
            ptr: 1,
        }
    }
}

#[derive(Clone)]
pub enum Argument {
    String(String),
    UInt(String),
    Int(String),
    Bool(String),
}

impl Argument {
    pub fn label(self: Argument) -> String {
        match self {
            Argument::String(d) => d,
            Argument::UInt(d) => d,
            Argument::Int(d) => d,
            Argument::Bool(d) => d,
        }
    }
}

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub function:
        Option<fn(Arc<HttpClient>, MessageCreate, Command) -> BoxFuture<anyhow::Result<()>>>,
    pub subcommands: Option<Box<[Command]>>,
    pub arguments: Option<Box<[Argument]>>,
}

impl Command {
    pub fn new(
        name: String,
        function: Option<
            fn(Arc<HttpClient>, MessageCreate, Command) -> BoxFuture<anyhow::Result<()>>,
        >,
        subcommands: &[Command],
        arguments: &[Argument],
    ) -> Command {
        return Command {
            name,
            function,
            subcommands: Some(subcommands.into()),
            arguments: Some(arguments.into()),
        };
    }

    pub fn find_command(self: Command, command: &str) -> Option<Box<Command>> {
        if let Some(sc) = &self.subcommands {
            for c in sc {
                if c.name == command {
                    return Some(Box::new((*c).clone()));
                }
            }
        };
        return None;
    }
}

pub fn get_message_command<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    let mut iter = message.split_ascii_whitespace();
    return iter.next().unwrap_or("").strip_prefix(prefix);
}
