use std::fs::File;
use std::io::{stdin, Read};
use std::path::Path;

use grep::matcher::Match;
use grep::regex::RegexMatcher;
use grep::searcher::Searcher;

use app::InputSource;
use error::MError;
use standard::StandardSink;

/// Search the given subject using the appropriate strategy.
pub fn search(
    matches: &mut Vec<(u64, Match)>,
    input_file: &InputSource,
    pattern: &str,
) -> Result<(), MError> {
    let matcher = match RegexMatcher::new(&pattern[..]) {
        Err(_) => return Err(MError::Error),
        Ok(m) => m,
    };

    let mut sink = StandardSink {
        matcher: matcher,
        matches: matches,
        match_count: 0,
    };

    return match input_file {
        InputSource::Stdin(file) => {
            let stdin = stdin();
            // A `return` here appeases the borrow checker. NLL will fix this.
            //return search_reader(&mut sink, stdin.lock());
            return search_file(&mut sink, file);
        }
        InputSource::File(file) => search_file(&mut sink, file),
    };
}

// For stdin
fn search_reader<R: Read>(
    sink: &mut StandardSink,
    reader: R,
) -> Result<(), MError> {
    return match Searcher::new().search_reader(
        sink.matcher.clone(),
        reader,
        sink,
    ) {
        Err(_) => Err(MError::Error),
        Ok(_) => Ok(()),
    };
}

fn search_file(sink: &mut StandardSink, file: &File) -> Result<(), MError> {
    return match Searcher::new().search_file(sink.matcher.clone(), file, sink) {
        Err(_) => Err(MError::Error),
        Ok(_) => Ok(()),
    };
}

fn search_path(sink: &mut StandardSink, path: &Path) -> Result<(), MError> {
    return match Searcher::new().search_path(sink.matcher.clone(), path, sink) {
        Err(_) => Err(MError::Error),
        Ok(_) => Ok(()),
    };
}
