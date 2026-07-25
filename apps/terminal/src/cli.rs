// SPDX-License-Identifier: MPL-2.0

use std::{ffi::OsString, path::PathBuf};

use anyhow::{Result, bail};
use raster_engine::{DirectLaunchRequest, GameId, RunSeed, StartupOptions};

const USAGE: &str = "usage:
  raster-nights [--quiet]
  raster-nights display-test
  raster-nights diagnostics [--output PATH]
  raster-nights validate-content
  raster-nights [--quiet] play <signal-stack|loopback> [--quick] [--seed N]";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Run(StartupOptions),
    DisplayTest,
    Diagnostics {
        output: Option<PathBuf>,
    },
    ValidateContent,
    #[cfg(debug_assertions)]
    TestPanicAfterTerminalInit,
}

pub(crate) fn parse_arguments() -> Result<Command> {
    parse_from(std::env::args_os())
}

fn parse_from(arguments: impl IntoIterator<Item = OsString>) -> Result<Command> {
    let mut arguments = arguments.into_iter();
    let _executable = arguments.next();
    let mut values = arguments
        .map(|argument| {
            argument
                .into_string()
                .map_err(|_| anyhow::anyhow!("arguments must be valid UTF-8\n{USAGE}"))
        })
        .collect::<Result<Vec<_>>>()?;

    let quiet = if values.first().is_some_and(|argument| argument == "--quiet") {
        values.remove(0);
        true
    } else {
        false
    };
    if values.is_empty() {
        return Ok(Command::Run(StartupOptions {
            quiet,
            direct_launch: None,
        }));
    }

    match values[0].as_str() {
        "display-test" if values.len() == 1 && !quiet => Ok(Command::DisplayTest),
        "validate-content" if values.len() == 1 && !quiet => Ok(Command::ValidateContent),
        "diagnostics" if !quiet => parse_diagnostics(&values[1..]),
        "play" => parse_play(&values[1..], quiet),
        #[cfg(debug_assertions)]
        "--test-panic-after-terminal-init" if values.len() == 1 && !quiet => {
            Ok(Command::TestPanicAfterTerminalInit)
        }
        _ => bail!(USAGE),
    }
}

fn parse_diagnostics(arguments: &[String]) -> Result<Command> {
    match arguments {
        [] => Ok(Command::Diagnostics { output: None }),
        [flag, path] if flag == "--output" && !path.is_empty() => Ok(Command::Diagnostics {
            output: Some(PathBuf::from(path)),
        }),
        _ => bail!(USAGE),
    }
}

fn parse_play(arguments: &[String], quiet: bool) -> Result<Command> {
    let Some(game) = arguments.first() else {
        bail!(USAGE);
    };
    if !matches!(game.as_str(), "signal-stack" | "loopback") {
        bail!("unknown or unavailable direct-launch game {game:?}\n{USAGE}");
    }

    let mut quick = false;
    let mut seed = None;
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--quick" if !quick => {
                quick = true;
                index += 1;
            }
            "--seed" if seed.is_none() && index + 1 < arguments.len() => {
                seed = Some(RunSeed(arguments[index + 1].parse::<u64>().map_err(
                    |_| anyhow::anyhow!("seed must be an unsigned 64-bit integer\n{USAGE}"),
                )?));
                index += 2;
            }
            _ => bail!(USAGE),
        }
    }

    Ok(Command::Run(StartupOptions {
        quiet,
        direct_launch: Some(DirectLaunchRequest {
            game_id: GameId::parse(game.clone())?,
            quick,
            seed,
        }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(arguments: &[&str]) -> Result<Command> {
        parse_from(arguments.iter().map(OsString::from))
    }

    #[test]
    fn parses_normal_quiet_and_display_test() {
        assert_eq!(
            parse(&["raster-nights"]).expect("normal"),
            Command::Run(StartupOptions::default())
        );
        assert_eq!(
            parse(&["raster-nights", "--quiet"]).expect("quiet"),
            Command::Run(StartupOptions {
                quiet: true,
                direct_launch: None,
            })
        );
        assert_eq!(
            parse(&["raster-nights", "display-test"]).expect("display test"),
            Command::DisplayTest
        );
    }

    #[test]
    fn parses_diagnostics_output() {
        assert_eq!(
            parse(&["raster-nights", "diagnostics", "--output", "report.txt"])
                .expect("diagnostics"),
            Command::Diagnostics {
                output: Some(PathBuf::from("report.txt"))
            }
        );
    }

    #[test]
    fn parses_direct_launch_options_in_either_order() {
        let expected = Command::Run(StartupOptions {
            quiet: true,
            direct_launch: Some(DirectLaunchRequest {
                game_id: GameId::parse("loopback").expect("ID"),
                quick: true,
                seed: Some(RunSeed(42)),
            }),
        });
        assert_eq!(
            parse(&[
                "raster-nights",
                "--quiet",
                "play",
                "loopback",
                "--quick",
                "--seed",
                "42"
            ])
            .expect("direct"),
            expected
        );
    }

    #[test]
    fn rejects_hidden_unknown_and_invalid_combinations() {
        assert!(parse(&["raster-nights", "play", "packet-sweep"]).is_err());
        assert!(parse(&["raster-nights", "play", "unknown"]).is_err());
        assert!(parse(&["raster-nights", "play", "loopback", "--seed"]).is_err());
        assert!(parse(&["raster-nights", "display-test", "--quiet"]).is_err());
        assert!(parse(&["raster-nights", "diagnostics", "--output"]).is_err());
    }
}
