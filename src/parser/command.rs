use std::{future::Future, pin::Pin, sync::Arc};

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
    pub fn new(mut t: TextCommand, c: Command) -> CommandWithData {
        CommandWithData {
            name: c.name,
            subcommand: if let Some(subcommands) = c.subcommands {
                let mut res = None;
                for subcommand in subcommands {
                    if let Some(word) = t.next() {
                        if subcommand.name == word {
                            res = Some(Box::new(CommandWithData::new(t.clone(), subcommand)));
                            break;
                        }
                    }
                }
                res
            } else {
                None
            },
            arguments: if let Some(arguments) = c.arguments {
                let mut collected_args: Vec<ArgumentWithData> = vec![];
                for argument in arguments {
                    if argument.clone().size() == 0 {
                        match argument {
                            Argument::String(_) => {
                                collected_args.push(ArgumentWithData::String(
                                    t.clone().collect::<Vec<String>>().join(" "),
                                ));
                            }
                            Argument::UInt(_) => {
                                let uints =
                                    t.clone().map(|x| x.parse().unwrap()).collect::<Vec<u32>>();
                                for uint in uints {
                                    collected_args.push(ArgumentWithData::UInt(uint));
                                }
                            }
                            Argument::Int(_) => {
                                let ints =
                                    t.clone().map(|x| x.parse().unwrap()).collect::<Vec<i32>>();
                                for int in ints {
                                    collected_args.push(ArgumentWithData::Int(int));
                                }
                            }
                            Argument::Bool(_) => {
                                let bools =
                                    t.clone().map(|x| x.parse().unwrap()).collect::<Vec<bool>>();
                                for bool in bools {
                                    collected_args.push(ArgumentWithData::Bool(bool));
                                }
                            }
                        }
                    } else {
                        match argument {
                            Argument::String(_) => {
                                if let Some(arg) = t.clone().next() {
                                    collected_args
                                        .push(ArgumentWithData::String(t.clone().next().unwrap()));
                                }
                            }
                            Argument::Int(_) => {
                                if let Some(arg) = t.clone().next() {
                                    collected_args
                                        .push(ArgumentWithData::Int(arg.parse().unwrap()));
                                }
                            }
                            Argument::UInt(_) => {
                                if let Some(arg) = t.clone().next() {
                                    collected_args
                                        .push(ArgumentWithData::UInt(arg.parse().unwrap()));
                                }
                            }
                            Argument::Bool(_) => {
                                if let Some(arg) = t.clone().next() {
                                    collected_args
                                        .push(ArgumentWithData::Bool(arg.parse().unwrap()));
                                }
                            }
                        }
                    }
                }
                Some(collected_args.into())
            } else {
                None
            },
        }
    }
}

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub function: Option<
        fn(Arc<State<'static>>, MessageCreate, CommandWithData) -> BoxFuture<anyhow::Result<()>>,
    >,
    pub subcommands: Option<Box<[Command]>>,
    pub arguments: Option<Box<[Argument]>>,
}

impl Command {
    pub fn new(
        name: String,
        function: Option<
            fn(
                Arc<State<'static>>,
                MessageCreate,
                CommandWithData,
            ) -> BoxFuture<anyhow::Result<()>>,
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
