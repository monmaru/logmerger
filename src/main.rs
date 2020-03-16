extern crate chrono;

use chrono::NaiveDateTime;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let v: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        match NaiveDateTime::parse_from_str(v[0], "%F %H:%M:%S,%3f") {
            Ok(dt) => println!("{:?}", dt),
            Err(err) => println!("{}", err.to_string()),
        }
    }
    Ok(())
}
