type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use derive_more::derive::Display;
use log::{debug, error, info, LevelFilter};

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

    debug!("Starting at log level {:?}", &log_level);

    let mut scan_flags = die::ScanFlags::DEEP_SCAN;

    if log_level >= LevelFilter::Debug {
        scan_flags |= die::ScanFlags::VERBOSE;
    }

    let res = match args.database {
        Some(db) => {
            debug!(
                "Scanning with flags {:?} and database path {}",
                scan_flags,
                args.filepath.as_path().to_string_lossy()
            );

            die::scan_file_with_db(args.filepath.as_path(), scan_flags, db.as_path())
        }
        None => {
            debug!("Scanning with flags {:?}", scan_flags);
            die::scan_file(args.filepath.as_path(), scan_flags)
        }
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
