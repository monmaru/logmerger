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

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Line {
    time: NaiveDateTime,
    content: String,
    priority: u8,
}

impl Line {
    pub fn new(time: NaiveDateTime, content: String, priority: u8) -> Self {
        Line {
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

fn read_log_file(path: &str, priority: u8) -> Result<Vec<Line>, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut ret: Vec<Line> = Vec::new();
    for content in reader.lines() {
        let content = content?;
        let v: Vec<&str> = content.split('|').map(|s| s.trim()).collect();
        match NaiveDateTime::parse_from_str(v[0], "%F %H:%M:%S,%3f") {
            Ok(dt) => {
                ret.push(Line::new(dt, content, priority));
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
        .arg(Arg::with_name("base").index(1).required(true))
        .arg(Arg::with_name("target").index(2).required(true))
        .arg(Arg::with_name("2nd target").index(3).required(false))
        .arg(Arg::with_name("3rd target").index(4).required(false))
        .get_matches();

    let base_path = matches.value_of("base").unwrap();
    let mut base_log = read_log_file(base_path, 1)?;
    let target_log = read_log_file(matches.value_of("target").unwrap(), 2)?;

    let start_time = base_log.first().unwrap().time;
    base_log.extend(target_log);

    if let Some(target) = matches.value_of("2nd target") {
        base_log.extend(read_log_file(target, 3)?);
    }
    if let Some(target) = matches.value_of("3rd target") {
        base_log.extend(read_log_file(target, 4)?);
    }

    base_log.sort_by(|a, b| a.time.cmp(&b.time).then(a.priority.cmp(&b.priority)));

    let file_stem = Path::new(base_path).file_stem().unwrap();
    let dest_path = Path::new(env::current_dir()?.to_str().unwrap())
        .join(format!("{}_merged.log", file_stem.to_str().unwrap()));
    let mut f = File::create(&dest_path)?;
    for line in base_log.iter().filter(|l| l.time >= start_time) {
        write!(f, "[{}] ", line.priority)?;
        write!(f, "{}", line.content)?;
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
