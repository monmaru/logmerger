extern crate chrono;

use chrono::NaiveDateTime;
use std::env;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Line {
    time: NaiveDateTime,
    content: String,
}

impl Line {
    pub fn new(time: NaiveDateTime, content: String) -> Self {
        Line { time, content }
    }
}

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let mut base = read_log_file(&args[1])?;
    let target = read_log_file(&args[2])?;
    base.extend(target);
    base.sort_by(|a, b| a.time.cmp(&b.time));

    let mut f = File::create("foo.txt")?;
    for l in base {
        f.write(l.content.as_bytes())?;
        f.write(LINE_ENDING.as_bytes())?;
    }
    Ok(())
}

fn read_log_file(path: &str) -> Result<Vec<Line>, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut ret: Vec<Line> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let v: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        match NaiveDateTime::parse_from_str(v[0], "%F %H:%M:%S,%3f") {
            Ok(dt) => {
                ret.push(Line::new(dt, line));
            }
            Err(_err) => {
                if let Some(last) = ret.last_mut() {
                    last.content += &(LINE_ENDING.to_owned() + &line);
                }
            }
        }
    }
    Ok(ret)
}
