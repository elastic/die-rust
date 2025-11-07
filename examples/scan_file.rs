type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

use std::{env, path::PathBuf};

use clap::{Parser, ValueEnum};
use derive_more::derive::Display;
use log::{LevelFilter, debug, error, info, trace};

#[derive(Default, Clone, Debug, Display, ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Csv,
    Xml,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct Args {
    #[arg(value_name = "FILE")]
    filepath: PathBuf,

    #[arg(short, long = "verbose", action = clap::ArgAction::Count)]
    verbosity: u8,

    #[arg(short, long = "database-path", value_name = "DATABASE")]
    database: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = match args.verbosity {
        2 => LevelFilter::Trace, // -vv
        1 => LevelFilter::Debug, // -v
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new().filter(None, log_level).init();
    trace!("env_logger initialized");

    debug!("Starting at log level {:?}", &log_level);

    let mut scan_flags = die::ScanFlags::DEEP_SCAN;

    if log_level >= LevelFilter::Debug {
        trace!("Setting VERBOSE scan flag");
        scan_flags |= die::ScanFlags::VERBOSE;
    }

    debug!("Scanning with flags {:?}", scan_flags,);

    let res = match args.database {
        Some(db) => {
            trace!(
                "Using database path {} from command line",
                args.filepath.as_path().to_string_lossy()
            );

            die::scan_file_with_db(args.filepath.as_path(), scan_flags, db.as_path())
        }
        None => match env::var("DIE_DB_PATH") {
            Ok(db_str) => {
                let db_path = PathBuf::from(db_str);
                trace!("Using database path from environment {:?}", &db_path);
                die::scan_file_with_db(args.filepath.as_path(), scan_flags, db_path.as_path())
            }
            Err(_) => {
                trace!("Scanning without database");
                die::scan_file(args.filepath.as_path(), scan_flags)
            }
        },
    };

    match res {
        Ok(sigmatch) => {
            info!("{}: {}", args.filepath.to_string_lossy(), sigmatch);
        }
        Err(e) => {
            error!(
                "scan_file() failed while scanning '{}', reason {:?}",
                args.filepath.to_string_lossy(),
                e
            );
        }
    }

    Ok(())
}
