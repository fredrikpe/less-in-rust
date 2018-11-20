
use std::env;
use std::error::Error;
use std::io;
use std::process;

use grep_regex::RegexMatcher;
use grep_searcher::Searcher;
use grep_searcher::sinks::UTF8;

pub fn example(reader: R) -> Result<(), Box<Error>> 
    where R: std::io::Read,
{
    let pattern = "update";

    let matcher = RegexMatcher::new(&pattern)?;
    Searcher::new().search_reader(&matcher, io::stdin(), UTF8(|lnum, line| {
        print!("{}:{}", lnum, line);
        Ok(true)
    }))?;
    Ok(())
}
