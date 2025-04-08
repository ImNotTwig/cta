mod fun;
mod music;
mod utility;

use crate::parser::Argument;
use crate::parser::ArgumentMetadata;
use crate::parser::Command;

pub use fun::jump;

pub use utility::ping;

pub use music::join;
pub use music::leave;

pub use music::next;
pub use music::prev;

pub use music::play;

pub use music::pause;
pub use music::unpause;

pub use music::insert;
pub use music::playnext;
pub use music::queue;
pub use music::remove;

pub fn rootcmd() -> Command {
    Command::new(
        String::from("cta"),
        None,
        &[
            Command::new(
                String::from("ping"),
                Some(ping),
                &[],
                &[Argument::String(ArgumentMetadata {
                    label: String::from("text"),
                    size: 0,
                })],
            ),
            Command::new(
                String::from("jump"),
                Some(jump),
                &[],
                &[Argument::UInt(ArgumentMetadata {
                    label: String::from("amount"),
                    size: 1,
                })],
            ),
            Command::new(String::from("join"), Some(join), &[], &[]),
            Command::new(String::from("leave"), Some(leave), &[], &[]),
            Command::new(String::from("pause"), Some(pause), &[], &[]),
            Command::new(String::from("unpause"), Some(unpause), &[], &[]),
            Command::new(
                String::from("play"),
                Some(play),
                &[],
                &[Argument::String(ArgumentMetadata {
                    label: String::from("song"),
                    size: 0,
                })],
            ),
            Command::new(
                String::from("playnext"),
                Some(playnext),
                &[],
                &[Argument::String(ArgumentMetadata {
                    label: String::from("song"),
                    size: 0,
                })],
            ),
            Command::new(String::from("next"), Some(next), &[], &[]),
            Command::new(String::from("prev"), Some(prev), &[], &[]),
            Command::new(String::from("queue"), Some(queue), &[], &[]),
            Command::new(
                String::from("remove"),
                Some(remove),
                &[],
                &[Argument::UInt(ArgumentMetadata {
                    label: String::from("index"),
                    size: 1,
                })],
            ),
            Command::new(
                String::from("insert"),
                Some(insert),
                &[],
                &[
                    Argument::UInt(ArgumentMetadata {
                        label: String::from("index"),
                        size: 1,
                    }),
                    Argument::String(ArgumentMetadata {
                        label: String::from("song"),
                        size: 0,
                    }),
                ],
            ),
        ],
        &[],
    )
}
