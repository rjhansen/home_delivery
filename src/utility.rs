use std::path::Path;
use std::process::exit;
use std::thread::sleep;
use chrono::{DateTime, Duration, Utc};
use log::{error, info};
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
        match entry {
            Err(e) => {
                error!("error reading source directory entry: {}", e);
                continue;
            },
            Ok(direntry) => {
                if !direntry.path().is_file() {
                    info!("Skipping non-file {}", direntry.path().display());
                    continue;
                }
                match direntry.path().file_name() {
                    None => {
                        error!("couldn’t get file name");
                        continue;
                    },
                    Some(file_name) => match file_name.to_str() {
                        None => {
                            error!("couldn’t convert file name to Unicode string");
                            continue;
                        }
                        Some(f) => {
                            match RE.captures(f) {
                                None => {
                                    continue;
                                },
                                Some(_) => {
                                    match direntry.path().to_str() {
                                        None => {
                                            error!("couldn’t convert dirpath to Unicode string");
                                            continue;
                                        },
                                        Some(g) => {
                                            info!("found {}", g);
                                            matching_filenames.push(g.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Step three: munge timestamps and create a data structure of filename and
    // delivery time
    for path in matching_filenames {
        let capture;
        match RE.captures(path.as_str()) {
            None => {
                continue;
            }
            Some(c) => {
                capture = c;
            }
        }
        let day;
        let hour;
        let minute;
        
        day = match capture.name("day") {
            Some(cap) => cap.as_str(),
            None => "0",
        };
        hour = match capture.name("hour") {
            Some(cap) => cap.as_str(),
            None => "00"
        };
        minute = match capture.name("minute") {
            Some(cap) => cap.as_str(),
            None => "00"
        };

        // sanity-check: a file with an invalid timestamp won't be picked up
        if hour.parse::<u32>().unwrap() < 24 && minute.parse::<u32>().unwrap() < 60 {
            let date = (Utc::now() + Duration::seconds(86400 * day.parse::<i64>().unwrap()))
                .format("%Y-%m-%d")
                .to_string();
            rv.push((
                path.to_string(),
                DateTime::parse_from_rfc3339(format!("{}T{}:{}:00Z", date, hour, minute).as_str())
                    .unwrap()
                    .to_utc()
            ));
        }
    };

    rv.sort_by(|a, b| a.1.cmp(&b.1));
    rv
}

pub fn deliver(filenames: Vec<(String, DateTime<Utc>)>, destination: &Path) {
    for (path, _) in filenames {
        let name = Path::new(path.as_str()).file_name().unwrap().to_str().unwrap();
        info!(
            "moving {} to {}",
            path.to_string(),
            destination.join(name).to_str().unwrap()
        );
        match std::fs::copy(path.as_str(), destination.join(name)) {
            Ok(_) => {}
            Err(e) => {
                error!("Error copying file: {}", e);
                exit(1);
            }
        }
        match std::fs::remove_file(path) {
            Ok(_) => {}
            Err(e) => {
                error!("Error removing file: {}", e);
                exit(1);
            }
        }
    }
}