mod utility;

use clap::Parser;
use log::info;
use log4rs;
use std::path::Path;
use std::process::exit;
use chrono::{DateTime, Utc};
use utility::{deliver, filenames_with_timestamps, sleep_to_top_of_minute, sanity_check};

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Delivers files from one directory to another on a schedule.")]
#[command(author = "Rob Hansen <rob@hansen.engineering>")]
#[command(help_template("\
{before-help}{name} {version}
by {author-with-newline}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"))]
struct Args {
    #[arg(short, long, long_help = "Path to source directory")]
    source: String,

    #[arg(short, long, long_help = "Path to destination directory")]
    destination: String,

    #[arg(short, long, long_help = "Path to logging configuration file")]
    config: String,
}

fn main() {
    let args = Args::parse();
    let source = Path::new(&args.source);
    let destination = Path::new(&args.destination);
    let config = &args.config;
    sanity_check(source, destination, config);
    log4rs::init_file(config, Default::default()).unwrap();

    info!("home_delivery is starting");
    loop {
        info!("polling {}", source.to_str().unwrap());
        let total_files = filenames_with_timestamps(source).len();
        let deliverables: Vec<(String, DateTime<Utc>)> = filenames_with_timestamps(source)
            .clone()
            .into_iter()
            .filter(|(_, date)| date < &Utc::now())
            .collect();
        if 0 == total_files {
            info!("no files left — exiting normally");
            exit(0);
        } else if 0 < deliverables.len() {
            info!("{} file(s), {} ready for delivery", total_files, deliverables.len());
            deliver(deliverables, destination);
        }
        sleep_to_top_of_minute();
    }
}
