use chrono::{TimeZone, Utc};
use clap::Parser;
use memory_lol::lookup::Lookup;
use simplelog::LevelFilter;
use std::io::BufRead;

fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    let _ = init_logging(opts.verbose)?;
    let db = Lookup::new(&opts.db)?;

    match opts.command {
        Command::Import => {
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                let line = line?;
                let parts = line.split(',').collect::<Vec<_>>();
                let user_id = parts
                    .get(0)
                    .and_then(|value| value.parse::<u64>().ok())
                    .ok_or_else(|| Error::InvalidImportLine(line.clone()))?;
                let screen_name = parts
                    .get(1)
                    .ok_or_else(|| Error::InvalidImportLine(line.clone()))?;

                let mut dates = vec![];

                for part in &parts[2..] {
                    let value = part
                        .parse::<i64>()
                        .map_err(|_| Error::InvalidImportLine(line.clone()))?;
                    dates.push(Utc.timestamp(value, 0).naive_utc().date());
                }

                dates.sort();
                dates.dedup();

                db.insert_pair(user_id, screen_name, dates)?;
            }
        }
        Command::LookupId { id } => {
            let result = db.lookup_by_user_id(id)?;
            let mut results = result.iter().collect::<Vec<_>>();
            results.sort_by_key(|(screen_name, _)| screen_name.to_string());

            for (screen_name, dates) in results {
                println!(
                    "{}: {}",
                    screen_name,
                    dates
                        .iter()
                        .map(|date| date.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        Command::Stats => {
            let (pair_count, user_id_count, screen_name_count) = db.get_counts()?;

            println!("Accounts: {}", user_id_count);
            println!("Screen names: {}", screen_name_count);
            println!("Pairs: {}", pair_count);
        }
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Application error")]
    App(#[from] memory_lol::error::Error),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Log initialization error")]
    LogInitialization(#[from] log::SetLoggerError),
    #[error("Invalid import line")]
    InvalidImportLine(String),
}

#[derive(Debug, Parser)]
#[clap(name = "manage", version, author)]
struct Opts {
    /// Level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    /// Database directory path
    #[clap(long)]
    db: String,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Import,
    LookupId {
        /// Twitter user ID
        id: u64,
    },
    Stats,
}

fn select_log_level_filter(verbosity: i32) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

/// Initialize a default terminal logger with the indicated log level.
pub fn init_logging(verbosity: i32) -> Result<(), log::SetLoggerError> {
    simplelog::TermLogger::init(
        select_log_level_filter(verbosity),
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )
}
