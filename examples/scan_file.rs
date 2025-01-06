type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use derive_more::derive::Display;
use log::{error, info, LevelFilter};

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
        4 => LevelFilter::Trace, // -vvv
        3 => LevelFilter::Debug, // -vv
        1 => LevelFilter::Debug, // -v
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new().filter(None, log_level).init();

    let res = match args.database {
        Some(db) => die::scan_file_with_db(
            args.filepath.as_path(),
            die::ScanFlags::DEEP_SCAN,
            db.as_path(),
        ),
        None => die::scan_file(args.filepath.as_path(), die::ScanFlags::DEEP_SCAN),
    };

    match res {
        Ok(s) => {
            info!("{}", s);
        }
        Err(e) => {
            error!("scan_file returned {:?}", e);
        }
    }

    Ok(())
}
