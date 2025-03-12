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

use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use log::{error, info, warn};
use regex::Regex;
use std::fs::{DirEntry, ReadDir};
use std::path::Path;
use std::process::exit;
use std::thread::sleep;

static REXP: &str = r"^.*(((?<day>\d+)[dD])?(?<hour>\d\d)(?<minute>\d\d))(.*)$";
lazy_static! {
    static ref RE: Regex = Regex::new(REXP).unwrap();
}

pub fn sanity_check(source: &Path, destination: &Path, config: &String) {
    let source_ok = source.exists() && source.is_dir();
    let destination_ok = destination.exists() && destination.is_dir();
    let config_ok = {
        let configpath = Path::new(config);
        configpath.exists() && configpath.is_file()
    };

    if !(source_ok && destination_ok && config_ok) {
        error!("Error: invalid parameters. Try '--help' for help.");
        exit(1);
    }
}

pub fn sleep_to_top_of_minute() {
    let now = Utc::now();
    let next_minute = now + Duration::minutes(1);
    let top_of = next_minute.format("%Y-%m-%dT%H:%M:00Z").to_string();
    if let Ok(future) = DateTime::parse_from_rfc3339(top_of.as_str()) {
        if let Ok(as_std) = (future.to_utc() - now).to_std() {
            sleep(as_std);
        } else {
            sleep(Duration::minutes(1).to_std().unwrap());
        }
    } else {
        sleep(Duration::minutes(1).to_std().unwrap());
    }
}

fn read_directory(path: &Path) -> ReadDir {
    match path.read_dir() {
        Ok(dir) => dir,
        Err(e) => {
            error!("Error reading source directory: {}", e);
            exit(1);
        }
    }
}

fn capture_to_rfc3339(capture: &regex::Captures) -> Option<DateTime<Utc>> {
    let day: &str;
    let hour: &str;
    let minute: &str;

    if let Some(d) = capture.name("day") {
        day = d.as_str();
    } else {
        day = "0";
    };

    if let Some(h) = capture.name("hour") {
        hour = h.as_str();
    } else {
        hour = "00";
    };

    if let Some(m) = capture.name("minute") {
        minute = m.as_str();
    } else {
        minute = "00";
    }

    // sanity-check: a file with an invalid timestamp won't be picked up
    if hour.parse::<u32>().unwrap() >= 24 || minute.parse::<u32>().unwrap() >= 60 {
        return None;
    }

    let days = day.parse::<i64>().unwrap_or(0);
    let duration = Duration::seconds(86400 * days);
    let date_string = (Utc::now() + duration).format("%Y-%m-%d").to_string();
    let rfc3339_string = format!("{}T{}:{}:00Z", date_string, hour, minute);
    Some(
        DateTime::parse_from_rfc3339(&rfc3339_string.as_str())
            .unwrap()
            .to_utc(),
    )
}

fn find_matching_files(contents: ReadDir) -> Vec<String> {
    let mut matching_filenames = Vec::<String>::new();
    let entries = contents
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .collect::<Vec<DirEntry>>();

    for dir_entry in entries {
        if let Some(filename) = dir_entry.file_name().to_str() {
            if let Some(_) = RE.captures(filename) {
                if let Some(good_name) = dir_entry.path().to_str() {
                    matching_filenames.push(good_name.to_string());
                } else {
                    info!("invalid character in file path: skipping");
                }
            } else {
                info!("file '{}' is not a deliverable", filename);
            }
        } else {
            warn!("invalid character in file name: skipping");
        }
    }

    matching_filenames
}

pub fn filenames_with_timestamps(src: &Path) -> Vec<(String, DateTime<Utc>)> {
    let mut rv: Vec<(String, DateTime<Utc>)> = vec![];

    for path in find_matching_files(read_directory(src)) {
        if let Some(capture) = RE.captures(&path) {
            if let Some(rfc3339) = capture_to_rfc3339(&capture) {
                rv.push((path, rfc3339));
            }
        }
    }
    rv.sort_by(|a, b| a.1.cmp(&b.1));
    rv
}

pub fn deliver(filenames: &Vec<String>, destination: &Path) {
    for path in filenames {
        if let Some(name_1) = Path::new(&path).file_name() {
            if let Some(name_2) = name_1.to_str() {
                if let Some(dest) = destination.join(name_2).to_str() {
                    info!("moving {} to {}", &path, dest);
                    if let Err(e) = std::fs::copy(&path, &dest) {
                        error!("Error copying file {}: {}", &path, e);
                    }
                    if let Err(e) = std::fs::remove_file(&path) {
                        error!("Error moving file {}: {}", &path, e);
                    }
                }
            }
        }
    }
}
