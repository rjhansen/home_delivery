use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use std::path::Path;
use std::process::exit;
use std::thread::sleep;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"^.*(((?<day>\d+)[dD])?(?<hour>\d\d)(?<minute>\d\d))(.*)$").unwrap();
}

pub fn sanity_check(source: &Path, destination: &Path, config: &String) {
    let source_ok = source.exists() && source.is_dir();
    let destination_ok = destination.exists() && destination.is_dir();
    let config_ok = {
        let configpath = Path::new(config);
        configpath.exists() && configpath.is_file()
    };

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
            .as_str(),
    )
    .unwrap()
    .to_utc();
    sleep(
        (next_minute - now)
            .to_std()
            .unwrap_or_else(|_| core::time::Duration::from_secs(60)),
    );
}

pub fn filenames_with_timestamps(src: &Path) -> Vec<(String, DateTime<Utc>)> {
    let mut rv: Vec<(String, DateTime<Utc>)> = vec![];
    let contents;
    let mut matching_filenames = vec![];

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
        let direntry;
        let filename;
        let matching_filename;

        // If we can't read the entry, just log it and continue to the
        // next. We make a best-effort, we-never-crash try.
        match entry {
            Err(e) => {
                error!("error reading source directory entry: {}", e);
                continue;
            }
            Ok(d) => {
                direntry = d;
            }
        }

        // If it's not a file, bounce to the next.
        if !direntry.path().is_file() {
            info!("Skipping non-file {}", direntry.path().display());
            continue;
        }

        // Can't recover the filename? Bounce to the next.
        match direntry.path().file_name() {
            None => {
                error!("couldn’t get file name");
                continue;
            }
            Some(x) => match x.to_str() {
                None => {
                    error!("couldn’t convert file name to Unicode string");
                    continue;
                }
                Some(f) => {
                    filename = f.to_string();
                }
            },
        }

        // Not a regex match? Bounce.
        match RE.captures(filename.as_str()) {
            None => {
                continue;
            }
            Some(_) => {}
        }

        // Finally: recover the matching filename, or bounce.
        match direntry.path().to_str() {
            None => {
                error!("couldn’t convert dirpath to Unicode string");
                continue;
            }
            Some(g) => {
                matching_filename = g.to_string();
                info!("found {}", g);
            }
        }
        matching_filenames.push(matching_filename);
    }

    // Step three: munge timestamps and create a data structure of filename and
    // delivery time
    for path in matching_filenames {
        let capture;
        let day;
        let hour;
        let minute;
        let hour_as_u32;
        let minute_as_u32;
        let days_duration;
        let date_tmp;
        let date;

        match RE.captures(path.as_str()) {
            None => {
                error!("wtf on capture? this shouldn’t happen");
                exit(1);
            }
            Some(c) => {
                capture = c;
            }
        }
        day = match capture.name("day") {
            Some(cap) => cap.as_str(),
            None => "0",
        };
        hour = match capture.name("hour") {
            Some(cap) => cap.as_str(),
            None => "00",
        };
        minute = match capture.name("minute") {
            Some(cap) => cap.as_str(),
            None => "00",
        };

        match hour.parse::<u32>() {
            Err(_) => {
                error!("wtf in hour parse? this shouldn’t happen");
                exit(1);
            }
            Ok(foo) => {
                hour_as_u32 = foo;
            }
        }
        match minute.parse::<u32>() {
            Err(_) => {
                error!("wtf in minute parse? this shouldn’t happen");
                exit(1);
            }
            Ok(foo) => {
                minute_as_u32 = foo;
            }
        }

        // sanity-check: a file with an invalid timestamp won't be picked up
        if !(hour_as_u32 < 24 && minute_as_u32 < 60) {
            continue;
        }

        match day.parse::<i64>() {
            Err(_) => {
                error!("wtf in day parse? this shouldn’t happen");
                exit(1);
            }
            Ok(d) => {
                days_duration = Duration::seconds(86400 * d);
            }
        }
        date_tmp = format!(
            "{}T{}:{}:00Z",
            (Utc::now() + days_duration).format("%Y-%m-%d").to_string(),
            hour,
            minute
        );
        match DateTime::parse_from_rfc3339(date_tmp.as_str()) {
            Err(_) => {
                error!("wtf in rfc3339 parse? this shouldn’t happen");
                exit(1);
            }
            Ok(d) => {
                date = d.to_utc();
            }
        }
        rv.push((path.to_string(), date));
    }

    rv.sort_by(|a, b| a.1.cmp(&b.1));
    rv
}

pub fn deliver(filenames: Vec<(String, DateTime<Utc>)>, destination: &Path) {
    for (path, _) in filenames {
        let name_osstr;
        let name_str;
        
        match Path::new(path.as_str()).file_name() {
            None => {
                info!("that’s weird, asked to deliver a file with non-Unicode name");
                continue;
            },
            Some(f) => { name_osstr = f;}
        }
        match name_osstr.to_str() {
            None => {
                info!("that’s weird part II, non-Unicode name");
                continue;
            },
            Some(f) => { name_str = f;}
        }
        
        info!("moving {} to {}", path.to_string(), destination.join(name_str).display());
        
        match std::fs::copy(path.as_str(), destination.join(name_str)) {
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
