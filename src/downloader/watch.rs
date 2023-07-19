use std::io::{BufRead, BufReader, Read};

use regex::Regex;

pub fn watch<R, F>(read: R, eof: u8, callback: F)
where
    R: Read,
    F: Fn(&str),
{
    let mut reader = BufReader::new(read);
    let mut buf = Vec::new();

    while let Ok(length) = reader.read_until(eof, &mut buf) {
        match length {
            0 => break,
            _ => {
                let result = String::from_utf8_lossy(&buf);
                let result = result.trim();
                callback(result);
                buf.clear();
            }
        }
    }
}


pub fn watch_progress<R, F>(read: R, callback: F)
where
    R: Read,
    F: Fn(f64) + Clone,
{
    let re: Regex = Regex::new(r"(?<progress>[0-9\.]*?)%").unwrap();
    watch(read, b'%', |result| {
        if let Some(caps) = re.captures(result) {
            let progress = caps.name("progress").unwrap();
            let progress  = progress.as_str().parse::<f64>().unwrap_or(-1.0) / 100.0;
            callback(progress);
        }
    });
}