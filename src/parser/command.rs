use std::{future::Future, pin::Pin};

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::State;

use super::{Argument, ArgumentWithData, TextCommand};

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

#[derive(Clone)]
pub struct CommandWithData {
    pub name: String,
    pub subcommand: Option<Box<CommandWithData>>,
    pub arguments: Option<Box<[ArgumentWithData]>>,
}

impl CommandWithData {
    pub fn new(mut t: TextCommand, c: Command) -> anyhow::Result<CommandWithData> {
        Ok(CommandWithData {
            name: c.name,
            subcommand: c.subcommands.and_then(|subcommands| {
                let mut res = None;
                for subcommand in subcommands {
                    if let Some(word) = t.next() {
                        if subcommand.name == word {
                            res = Some(Box::new(CommandWithData::new(t.clone(), subcommand).ok()?));
                            break;
                        }
                    }
                }
                res
            }),
            arguments: c.arguments.and_then(|arguments| {
                let mut collected_args: Vec<ArgumentWithData> = vec![];
                for argument in arguments {
                    if argument.clone().size() == 0 {
                        for arg in t.clone().into_iter() {
                            collected_args.push(ArgumentWithData::new(&argument, arg).ok()?);
                        }
                    } else {
                        if let Some(arg) = t.next() {
                            collected_args.push(ArgumentWithData::new(&argument, arg).ok()?);
                        }
                    }
                }
                Some(collected_args.into())
            }),
        })
    }
}

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub function:
        Option<fn(State, MessageCreate, CommandWithData) -> BoxFuture<anyhow::Result<()>>>,
    pub subcommands: Option<Box<[Command]>>,
    pub arguments: Option<Box<[Argument]>>,
}

impl Command {
    pub fn new(
        name: String,
        function: Option<
            fn(State, MessageCreate, CommandWithData) -> BoxFuture<anyhow::Result<()>>,
        >,
        subcommands: &[Self],
        arguments: &[Argument],
    ) -> Self {
        Self {
            name,
            function,
            subcommands: Some(subcommands.into()),
            arguments: Some(arguments.into()),
        }
    }

    pub fn find_command(&self, command: &str) -> Option<Box<Self>> {
        if let Some(sc) = &self.subcommands {
            for c in sc {
                if c.name == command {
                    return Some(Box::new((*c).clone()));
                }
            }
        }
        None
    }
}
