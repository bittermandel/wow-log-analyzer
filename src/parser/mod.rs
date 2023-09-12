mod cell;

use std::io::{BufRead, BufReader, Read};

use anyhow;
use nom::combinator::map;
use nom::error::ErrorKind;
use nom::sequence::tuple;
use nom::Err;
use nom::{bytes::complete::tag, character::complete::digit1, sequence::separated_pair, IResult};
use thiserror::Error;

use self::cell::{parse_log_csv, LogCell, LogEventDateTime, LogRow};

pub struct Parser<'a> {
    lines: Vec<String>,
    parsed_lines: Vec<Vec<LogCell<'a>>>,
}

impl Parser<'_> {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            parsed_lines: Vec::new(),
        }
    }

    pub fn parse_file(&self, file: String) {
        let time_start = std::time::Instant::now();
        let file = std::fs::File::open(file).expect("Could not open file");
        let reader = BufReader::new(file);

        let mut num_lines = 0;

        for line in reader.lines() {
            num_lines += 1;
            let strline = line.unwrap();
            let (remainder, _, row) = parse_line(strline.as_str());
            if remainder != "" {
                if row != LogRow::NotSupported {
                    println!(
                        "Failed to parse remainder: {}. Last cell: {:?}. Row: {:?}",
                        remainder, row, strline
                    );
                }
            }
        }

        println!("Parsed {} lines in {:?}", num_lines, time_start.elapsed());
    }
}

fn parse_line(input: &str) -> (&str, LogEventDateTime, LogRow) {
    let parsed_input = separated_pair(parse_date_time, tag("  "), parse_log_csv)(input);
    if parsed_input.is_err() {
        panic!("Failed to parse input: {:?}: {:?}", input, parsed_input);
    }
    let (remainder, result) = parsed_input.unwrap();

    (remainder, result.0, result.1)
}

fn parse_date(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(digit1, tag("/"), digit1)(input)
}

fn parse_time(input: &str) -> IResult<&str, (&str, &str, &str, &str, &str, &str, &str)> {
    tuple((digit1, tag(":"), digit1, tag(":"), digit1, tag("."), digit1))(input)
}

fn parse_date_time(input: &str) -> IResult<&str, LogEventDateTime> {
    let parser = separated_pair(parse_date, tag(" "), parse_time);

    map(parser, |(date, time)| LogEventDateTime {
        month: date.0,
        day: date.1,
        hour: time.0,
        minute: time.2,
        second: time.4,
        ms: time.6,
    })(input)
}
