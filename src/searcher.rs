use std::fs::File;
use std::io::{stdin, Read};
use std::path::Path;

use grep::matcher::Match;
use grep::regex::RegexMatcher;
use grep::searcher::Searcher;

use error::{Error, Result};
use reader::InputReader;
use standard::StandardSink;

// For stdin
pub fn search_reader<R: Read>(
    sink: &mut StandardSink,
    reader: R,
) -> Result<()> {
    return match Searcher::new().search_reader(
        sink.matcher.clone(),
        reader,
        sink,
    ) {
        Err(_) => Err(Error::SearchError),
        Ok(_) => Ok(()),
    };
}

pub fn search_file(sink: &mut StandardSink, file: &File) -> Result<()> {
    return match Searcher::new().search_file(sink.matcher.clone(), file, sink) {
        Err(_) => Err(Error::SearchError),
        Ok(_) => Ok(()),
    };
}

pub fn search_path(sink: &mut StandardSink, path: &Path) -> Result<()> {
    return match Searcher::new().search_path(sink.matcher.clone(), path, sink) {
        Err(_) => Err(Error::SearchError),
        Ok(_) => Ok(()),
    };
}
