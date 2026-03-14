mod config;
mod fun;
mod music;
mod utility;

use crate::parser::Argument;
use crate::parser::ArgumentMetadata;
use crate::parser::Command;

pub fn rootcmd() -> Command {
    Command::new(
        String::from("cta"),
        None,
        &[
            Command::new(
                String::from("ping"),
                Some(utility::ping),
                &[],
                &[Argument::String(ArgumentMetadata {
                    label: String::from("text"),
                    size: 0,
                })],
            ),
            Command::new(
                String::from("jump"),
                Some(fun::jump),
                &[],
                &[Argument::UInt(ArgumentMetadata {
                    label: String::from("amount"),
                    size: 1,
                })],
            ),
            Command::new(
                String::from("prefix"),
                Some(config::prefix),
                &[],
                &[Argument::String(ArgumentMetadata {
                    label: String::from("new prefix"),
                    size: 1,
                })],
            ),
            Command::new(String::from("join"), Some(music::join), &[], &[]),
            Command::new(String::from("leave"), Some(music::leave), &[], &[]),
            // Command::new(String::from("pause"), Some(pause), &[], &[]),
            // Command::new(String::from("unpause"), Some(unpause), &[], &[]),
            // Command::new(
            //     String::from("play"),
            //     Some(play),
            //     &[],
            //     &[Argument::String(ArgumentMetadata {
            //         label: String::from("song"),
            //         size: 0,
            //     })],
            // ),
            // Command::new(
            //     String::from("playnext"),
            //     Some(playnext),
            //     &[],
            //     &[Argument::String(ArgumentMetadata {
            //         label: String::from("song"),
            //         size: 0,
            //     })],
            // ),
            // Command::new(String::from("next"), Some(next), &[], &[]),
            // Command::new(String::from("prev"), Some(prev), &[], &[]),
            // Command::new(String::from("queue"), Some(queue), &[], &[]),
            // Command::new(
            //     String::from("remove"),
            //     Some(remove),
            //     &[],
            //     &[Argument::UInt(ArgumentMetadata {
            //         label: String::from("index"),
            //         size: 1,
            //     })],
            // ),
            // Command::new(
            //     String::from("insert"),
            //     Some(insert),
            //     &[],
            //     &[
            //         Argument::UInt(ArgumentMetadata {
            //             label: String::from("index"),
            //             size: 1,
            //         }),
            //         Argument::String(ArgumentMetadata {
            //             label: String::from("song"),
            //             size: 0,
            //         }),
            //     ],
            // ),
        ],
        &[],
    )
}
