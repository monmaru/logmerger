extern crate chrono;

use chrono::NaiveDateTime;
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
                    last.content += &(LINE_ENDING.to_owned() + &content);
                }
            }
        }
    }
    Ok(ret)
}

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let mut base_log = read_log_file(&args[1], 1)?;
    let target_log = read_log_file(&args[2], 2)?;
    base_log.extend(target_log);
    base_log.sort_by(|a, b| a.time.cmp(&b.time).then(a.priority.cmp(&b.priority)));

    let file_stem = Path::new(&args[1]).file_stem().unwrap();
    let mut f = File::create(format!("{}_merged.log", file_stem.to_str().unwrap()))?;
    for line in base_log {
        f.write(format!("[{}] ", line.priority).as_bytes())?;
        f.write(line.content.as_bytes())?;
        f.write(LINE_ENDING.as_bytes())?;
    }
    Ok(())
}
