use std::path::Path;
use std::process::exit;
use std::thread::sleep;
use chrono::{DateTime, Duration, Utc};
use log::{error, info, warn};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"^.*(((?<day>\d+)[dD])?(?<hour>\d\d)(?<minute>\d\d))(.*)$").unwrap();
}

pub fn sanity_check(source: &Path, destination: &Path, config: &String) {
    let source_ok = source.exists() && source.is_dir();
    let destination_ok = destination.exists() && destination.is_dir();
    let config_ok = Path::new(config).exists() && Path::new(config).is_file();
    
    if !(source_ok && destination_ok && config_ok) {
        error!("Error: invalid parameters. Try “--help” for help.");
        exit(1);
    }
}

pub fn sleep_to_top_of_minute() {
    let now = Utc::now();
    let next_minute = DateTime::parse_from_rfc3339(
        (now + Duration::minutes(1))
            .format("%Y-%m-%dT%H:%M:00Z")
            .to_string()
            .as_str())
        .unwrap()
        .to_utc();
    let duration = (next_minute - now).to_std().unwrap();
    sleep(duration);
}

pub fn filenames_with_timestamps(src: &Path) -> Vec<(String, DateTime<Utc>)> {
    let mut rv: Vec<(String, DateTime<Utc>)> = vec!();
    let contents;
    let mut matching_filenames = vec!();

    // Step one: ensure the directory is readable.
    match src.read_dir() {
        Ok(dir) => {
            contents = dir;
        }
        Err(e) => {
            error!("Error reading source directory: {}", e);
            exit(1);
        }
    }

    // Step two: collect filenames matching the regex
    for entry in contents {
        if let Ok(dir_entry) = entry {
            if dir_entry.path().is_file() {
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
            } else {
                info!("skipping a non-file");
            }
        } else {
            warn!("error getting directory entry");
        }
    }

    // Step three: munge timestamps and create a data structure of filename and
    // delivery time
    for path in matching_filenames {
        if let Some(capture) = RE.captures(&path) {
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
                continue;
            }
            let days = day.parse::<i64>().unwrap_or(0);
            let duration = Duration::seconds(86400 * days);
            let date_string = (Utc::now() + duration)
                .format("%Y-%m-%d")
                .to_string();
            let rfc3339_string = format!("{}T{}:{}:00Z",
                                      date_string,
                                      hour,
                                      minute);
            let rfc3339 =
                DateTime::parse_from_rfc3339(&rfc3339_string.as_str())
                    .unwrap()
                    .to_utc();
            rv.push((path.to_string(), rfc3339));
        }
    }
    rv.sort_by(|a, b| a.1.cmp(&b.1));
    rv
}

pub fn deliver(filenames: Vec<String>, destination: &Path) {
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