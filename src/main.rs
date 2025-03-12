/* REMEMBER TO STAGE YOUR CHANGES BACK INTO THE NOWEB SOURCE!
 *
 * Copyright 2025, Robert J. Hansen <rob@hansen.engineering>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License. */

mod utility;

use chrono::{DateTime, Utc};
use clap::Parser;
use log::info;
use log4rs;
use std::path::Path;
use std::process::exit;
use utility::{deliver, filenames_with_timestamps, sanity_check, sleep_to_top_of_minute};

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Delivers files from one directory to another on a schedule.")]
#[command(author = "Rob Hansen <rob@hansen.engineering>")]
#[command(before_help = "                    ** NOTE: ALL TIMES ARE IN UTC **\
    ")]
#[command(after_help = "                    ** NOTE: ALL TIMES ARE IN UTC **\
    ")]
#[command(help_template(
    "\
    {before-help}{name} {version}
    Copyright (c) 2025 by {author}
    Homepage: https://github.com/rjhansen/home_delivery

    This is open source software distributed under terms of the Apache
    Software Foundation's Apache-2.0 license. For the full text of the
    license, please see https://www.apache.org/licenses/LICENSE-2.0.html

    {about-with-newline}
    {usage-heading} {usage}

    {all-args}{after-help}
    "
))]
struct Args {
    #[arg(short, long, long_help = "Path to source directory")]
    source: String,

    #[arg(short, long, long_help = "Path to destination directory")]
    destination: String,

    #[arg(short, long, long_help = "Path to logging configuration file")]
    config: String,
}

fn get_current_deliverables(deliv: &Vec<(String, DateTime<Utc>)>) -> Vec<String> {
    deliv
        .clone()
        .into_iter()
        .filter(|(_, date)| date < &Utc::now())
        .map(|(path, _)| path)
        .collect::<Vec<String>>()
}
fn main() {
    let args = Args::parse();
    let source = Path::new(&args.source);
    let destination = Path::new(&args.destination);
    let config = &args.config;
    let source_str: &str;
    sanity_check(source, destination, config);
    source_str = source.to_str().unwrap_or_else(|| {
        eprintln!("Error: source path is not a valid UTF-8 string");
        exit(1);
    });
    log4rs::init_file(config, Default::default()).expect("couldn't init logger!");
    info!("home_delivery is starting");
    loop {
        info!("polling {:?}", source_str);
        let all_deliverables = filenames_with_timestamps(source);
        let deliver_now = get_current_deliverables(&all_deliverables);
        if !deliver_now.is_empty() {
            info!("{} file(s) ready for delivery", deliver_now.len());
            deliver(&deliver_now, destination);
        }
        sleep_to_top_of_minute();
    }
}
