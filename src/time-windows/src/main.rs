use std::cmp;
use std::env;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::PathBuf;

use anyhow::{bail, Result};
use regex::Regex;

fn main() -> Result<()> {
    let path = env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| panic!("path to search is required"));
    let the_file = PathBuf::from(path.clone());
    if !the_file.is_file() {
        bail!("{} is not a file", path);
    }

    let re = Regex::new("message_time=(?P<message_time>[^T]*)T[^ ]* message_first_seen=(?P<first_seen>[^T]*)T[^ ]* max_seen_time=(?P<max_seen>[^T]*)T[^ ]*")?;

    let mut maxmin = MaxMin::new();

    let mut line = 0;
    let mut buf = String::new();
    let mut lines = StrBufReader::open(the_file)?;
    while let Some(res) = lines.read_into(&mut buf) {
        res?;
        line += 1;
        if let Some(cap) = re.captures(&buf) {
            let mt = cap.name("message_time").unwrap();
            let first_seen = cap.name("first_seen").unwrap();
            let max_seen = cap.name("max_seen").unwrap();
            let mut updated = false;
            maxmin.max_ever = max(
                max(maxmin.max_ever, mt.as_str(), &mut updated),
                max_seen.as_str(),
                &mut updated,
            );
            maxmin.min_ever = min(
                min(maxmin.min_ever, mt.as_str(), &mut updated),
                first_seen.as_str(),
                &mut updated,
            );

            maxmin.max_message = max(maxmin.max_message, mt.as_str(), &mut updated);
            maxmin.min_message = min(maxmin.min_message, mt.as_str(), &mut updated);

            maxmin.max_first_seen = max(maxmin.max_first_seen, first_seen.as_str(), &mut updated);
            maxmin.min_first_seen = min(maxmin.min_first_seen, first_seen.as_str(), &mut updated);

            maxmin.max_max_seen = max(maxmin.max_max_seen, max_seen.as_str(), &mut updated);
            maxmin.min_max_seen = min(maxmin.min_max_seen, max_seen.as_str(), &mut updated);

            if updated {
                println!("line {} {:?}", line, maxmin)
            } else if line % 1_000_000 == 0 {
                println!("line {}", line);
            }
        }
    }

    println!("total lines={} {:?}", line, maxmin);

    Ok(())
}

#[derive(Debug)]
struct MaxMin {
    max_ever: String,
    min_ever: String,
    max_message: String,
    min_message: String,
    max_first_seen: String,
    min_first_seen: String,
    max_max_seen: String,
    min_max_seen: String,
}

impl MaxMin {
    fn new() -> MaxMin {
        MaxMin {
            max_ever: "0000".into(),
            min_ever: "9999".into(),
            max_message: "0000".into(),
            min_message: "9999".into(),
            max_first_seen: "0000".into(),
            min_first_seen: "9999".into(),
            max_max_seen: "0000".into(),
            min_max_seen: "9999".into(),
        }
    }
}

fn max(left: String, right: &str, updated: &mut bool) -> String {
    match (&*left).cmp(right) {
        cmp::Ordering::Greater => left,
        cmp::Ordering::Equal => left,
        cmp::Ordering::Less => {
            *updated = true;
            right.into()
        }
    }
}

fn min(left: String, right: &str, updated: &mut bool) -> String {
    match (&*left).cmp(right) {
        cmp::Ordering::Greater => {
            *updated = true;
            right.into()
        }
        cmp::Ordering::Equal => left,
        cmp::Ordering::Less => left,
    }
}

struct StrBufReader {
    reader: io::BufReader<File>,
}

impl StrBufReader {
    fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        Ok(Self { reader })
    }

    fn read_into<'buf>(&mut self, buffer: &'buf mut String) -> Option<io::Result<()>> {
        buffer.clear();

        self.reader
            .read_line(buffer)
            .map(|u| if u == 0 { None } else { Some(()) })
            .transpose()
    }
}
