extern crate chrono;
#[macro_use]
extern crate clap;

use ansi_term::Colour::Green;
use chrono::NaiveDateTime;
use clap::{App, Arg};
use std::env;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Priority {
    First = 1,
    Second,
    Third,
    Fourth,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Log {
    time: NaiveDateTime,
    content: String,
    priority: Priority,
}

impl Log {
    pub fn new(time: NaiveDateTime, content: String, priority: Priority) -> Self {
        Log {
            time,
            content,
            priority,
        }
    }
}

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

fn parse_log_file(path: &str, priority: Priority) -> Result<Vec<Log>, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut ret: Vec<Log> = Vec::new();
    for content in reader.lines() {
        let content = content?;
        let v: Vec<&str> = content.split('|').map(|s| s.trim()).collect();
        match NaiveDateTime::parse_from_str(v[0], "%F %H:%M:%S,%3f") {
            Ok(dt) => {
                ret.push(Log::new(dt, content, priority));
            }
            Err(_) => {
                if let Some(last) = ret.last_mut() {
                    last.content += &(LINE_ENDING.to_string() + &content);
                }
            }
        }
    }
    Ok(ret)
}

fn main() -> Result<(), io::Error> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("base")
                .index(1)
                .required(true)
                .help("Sets a base log file path"),
        )
        .arg(
            Arg::with_name("target")
                .index(2)
                .required(true)
                .help("Sets a target log file path"),
        )
        .arg(Arg::with_name("2nd target").index(3).required(false))
        .arg(Arg::with_name("3rd target").index(4).required(false))
        .arg(
            Arg::with_name("noidx")
                .help("Don't output index")
                .long("noidx"),
        )
        .get_matches();

    let base_path = matches.value_of("base").unwrap();
    let mut base_log = parse_log_file(base_path, Priority::First)?;
    let target_log = parse_log_file(matches.value_of("target").unwrap(), Priority::Second)?;

    let start_time = base_log.first().unwrap().time;
    base_log.extend(target_log);

    if let Some(target) = matches.value_of("2nd target") {
        base_log.extend(parse_log_file(target, Priority::Third)?);
    }
    if let Some(target) = matches.value_of("3rd target") {
        base_log.extend(parse_log_file(target, Priority::Fourth)?);
    }

    base_log.sort_by(|a, b| a.time.cmp(&b.time).then(a.priority.cmp(&b.priority)));

    let file_stem = Path::new(base_path).file_stem().unwrap();
    let dest_path = Path::new(env::current_dir()?.to_str().unwrap())
        .join(format!("{}_merged.log", file_stem.to_str().unwrap()));
    let mut f = File::create(&dest_path)?;
    for log in base_log.iter().filter(|l| l.time >= start_time) {
        if !matches.is_present("noidx") {
            write!(f, "[{}] ", log.priority as u8)?;
        }
        write!(f, "{}", log.content)?;
        write!(f, "{}", LINE_ENDING)?;
    }

    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support();

    println!("Merge succeeded!!!");
    println!(
        "[{}] was created.",
        Green.bold().paint(dest_path.to_str().unwrap())
    );
    Ok(())
}
