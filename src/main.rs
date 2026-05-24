use clap::{Arg, App};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};


type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    lines: usize,
    words: usize,
    bytes: usize,
    chars: usize,
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut lines = 0;
    let mut words = 0;
    let mut bytes = 0;
    let mut chars = 0;
    let mut line = String::new();

    loop {
	let b = file.read_line(&mut line)?;
	if b == 0 {
	    break;
	}
	bytes += b;
	lines += 1;
	words += line.split_whitespace().count();
	chars += line.chars().count();
	line.clear();
    }

    Ok(FileInfo {
	lines, words, bytes, chars
    })
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
	let text = "I don't want the world, I just want your half.\r\n";
	let info = count(Cursor::new(text));
	assert!(info.is_ok());
	let expected = FileInfo{
	    lines:1,
	    words:10,
	    bytes:48,
	    chars:48,
	};
	assert_eq!(info.unwrap(), expected);
    }
}

fn main() {
    if let Err(e) = get_args().and_then(run) {
	eprintln!("{}", e);
	std::process::exit(1);
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
	.version("0.1.0")
	.about("Rust wc")
	.arg(
	    Arg::with_name("files")
		.value_name("FILE")
		.help("Input files(s)")
		.default_value("-")
		.multiple(true),
	)
	.arg(
	    Arg::with_name("words")
		.short("w")
		.long("words")
		.help("Show word count")
		.takes_value(false),
	)
	.arg(
	    Arg::with_name("bytes")
		.short("c")
		.long("bytes")
		.help("Show byte count")
		.takes_value(false),
	)
	.arg(
	    Arg::with_name("chars")
		.short("m")
		.long("chars")
		.help("Show character count")
		.takes_value(false)
		.conflicts_with("bytes"),
	)
	.arg(
	    Arg::with_name("lines")
		.short("l")
		.long("lines")
		.help("Show line count")
		.takes_value(false),
	)
	.get_matches();

    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    if [lines, words, bytes, chars].iter().all(|v| v == &false) {
	lines = true;
	words = true;
	bytes = true;
    }

    Ok(Config {
	files: matches.values_of_lossy("files").unwrap(),
	lines,
	words,
	bytes,
	chars,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
	"-" => Ok(Box::new(BufReader::new(io::stdin()))),
	_ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn format_field(value: usize, show: bool) -> String {
    if show {
	format!("{:>8}", value)
    } else {
	"".to_string()
    }
}

fn run(config: Config) -> MyResult<()> {

    let mut lines = 0;
    let mut words = 0;
    let mut bytes = 0;
    let mut chars = 0;
    
    for filename in &config.files {
	match open(filename) {
	    Err(err) => eprintln!("{}: {}", filename, err),
	    Ok(file) => {
		if let Ok(info) = count(file) {
		    println!(
			"{}{}{}{}{}",
			format_field(info.lines, config.lines),
			format_field(info.words, config.words),
			format_field(info.bytes, config.bytes),
			format_field(info.chars, config.chars),
			if filename.as_str() == "-" {
			    "".to_string()
			} else {
			    format!(" {}", filename)
			}
		    );

		    lines += info.lines;
		    words += info.words;
		    bytes += info.bytes;
		    chars += info.chars;
			    
		}
	    }
	}
    }

    if config.files.len() > 1 {
	println!(
	    "{}{}{}{} total",
	    format_field(lines, config.lines),
	    format_field(words, config.words),
	    format_field(bytes, config.bytes),
	    format_field(chars, config.chars),
	);
    }
    
    Ok(())
}
