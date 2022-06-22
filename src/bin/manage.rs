use chrono::{TimeZone, Utc};
use clap::Parser;
use memory_lol::{
    import::{Session, UpdateMode},
    lookup::Lookup,
};
use simplelog::LevelFilter;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use zstd::stream::read::Decoder;

fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    let _ = init_logging(opts.verbose)?;
    let db = Lookup::new(&opts.db)?;

    match opts.command {
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
        Command::ImportMentions { input, zst } => {
            let file = File::open(input)?;

            let source: Box<dyn Read> = if zst {
                Box::new(Decoder::new(file)?)
            } else {
                Box::new(file)
            };

            let session = Session::load_mentions(source)?;
            let count = session.update(&db, UpdateMode::Range)?;

            log::info!("Updated {} entries", count);
        }
        Command::ImportJson { input, zst } => {
            let file = File::open(input)?;

            let source: Box<dyn Read> = if zst {
                Box::new(Decoder::new(file)?)
            } else {
                Box::new(file)
            };

            let reader = BufReader::new(source);

            let session = Session::load_json(reader)?;
            let count = session.update(&db, UpdateMode::Range)?;

            log::info!("Updated {} entries", count);
        }
        Command::ImportBatch { input, prefix } => {
            let prefix = prefix.as_ref();

            let mut paths = std::fs::read_dir(&input)?
                .filter_map(|entry| {
                    entry.map_or_else(
                        |error| Some(Err(error)),
                        |entry| {
                            let path = entry.path();

                            if prefix
                                .map(|prefix| {
                                    path.file_name()
                                        .and_then(|file_name| file_name.to_str())
                                        .map(|file_name| file_name.starts_with(prefix))
                                        .unwrap_or(false)
                                })
                                .unwrap_or(true)
                            {
                                Some(Ok(path))
                            } else {
                                None
                            }
                        },
                    )
                })
                .collect::<Result<Vec<_>, std::io::Error>>()?;
            paths.sort();

            for directory in paths {
                log::info!("Importing directory: {}", directory.to_string_lossy());

                let names_file_zst = directory.join("names.csv.zst");
                let names_file = directory.join("names.csv");

                let profiles_file_zst = directory.join("profiles.csv.zst");
                let profiles_file = directory.join("profiles.csv");

                let names_source: Option<Box<dyn Read>> = if names_file_zst.exists() {
                    let file = File::open(names_file_zst)?;
                    Some(Box::new(Decoder::new(file)?))
                } else if names_file.exists() {
                    let file = File::open(names_file)?;
                    Some(Box::new(file))
                } else {
                    None
                };

                let profiles_source: Option<Box<dyn Read>> = if profiles_file_zst.exists() {
                    let file = File::open(profiles_file_zst)?;
                    Some(Box::new(Decoder::new(file)?))
                } else if profiles_file.exists() {
                    let file = File::open(profiles_file)?;
                    Some(Box::new(file))
                } else {
                    None
                };

                let mut count = 0;

                if let Some(source) = names_source {
                    log::info!("Importing mentions");
                    let session = Session::load_mentions(source)?;
                    count += session.update(&db, UpdateMode::Range)?;
                }

                if let Some(source) = profiles_source {
                    log::info!("Importing profiles");
                    let reader = BufReader::new(source);
                    let session = Session::load_json(reader)?;
                    count += session.update(&db, UpdateMode::Range)?;
                }

                log::info!("Updated {} entries", count);
            }
        }
        Command::ImportMulti => {
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
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Application error")]
    App(#[from] memory_lol::error::Error),
    #[error("Import error")]
    Import(#[from] memory_lol::import::Error),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
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
    /// Look up a Twitter user ID in the database
    LookupId {
        /// Twitter user ID
        id: u64,
    },
    /// Print account, screen name, and pair counts
    Stats,
    /// Import a CSV file containing mentions
    ImportMentions {
        /// NDJSON file path
        #[clap(long)]
        input: String,
        /// Use ZSTD compression
        #[clap(long)]
        zst: bool,
    },
    /// Import an NDJSON file
    ImportJson {
        /// NDJSON file path
        #[clap(long)]
        input: String,
        /// Use ZSTD compression
        #[clap(long)]
        zst: bool,
    },
    /// Import a batch of Twitter Stream Grab output directories
    ImportBatch {
        /// Base directory
        #[clap(long)]
        input: String,
        /// Directory prefix
        #[clap(long)]
        prefix: Option<String>,
    },
    /// Import a CSV from stdin with multiple timestamps per row
    ImportMulti,
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
