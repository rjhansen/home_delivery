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
#[command(before_help = "                    ** NOTE: ALL TIMES ARE IN UTC **\
")]
#[command(after_help = "                    ** NOTE: ALL TIMES ARE IN UTC **\
")]
#[command(help_template("\
{before-help}{name} {version}
Copyright © 2025 by {author}
Homepage: https://github.com/rjhansen/home_delivery
Report bugs at: https://github.com/rjhansen/home_delivery/issues

This is open source software distributed under terms of the Apache Software
Foundation’s Apache-2.0 license. For the full text of the license, please
see https://www.apache.org/licenses/LICENSE-2.0.html

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
