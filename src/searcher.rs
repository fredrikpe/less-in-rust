use std::path::Path;
use std::io::{Read, stdin};

use grep::matcher::Match;
use grep::regex::RegexMatcher;
use grep::searcher::Searcher;

use error::MError;
use app::Input;
use standard::StandardSink;


/// Search the given subject using the appropriate strategy.
pub fn search(
    matches: &mut Vec<(u64, Match)>,
    input_file: &Input,
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
        Input::StdIn => {
            let stdin = stdin();
            // A `return` here appeases the borrow checker. NLL will fix this.
            return search_reader(&mut sink, stdin.lock());
        }
        Input::File(path) => search_path(&mut sink, std::path::Path::new(path))
    }
}

// For stdin
fn search_reader<R: Read>(sink: &mut StandardSink, reader: R) -> Result<(), MError> {
    return match Searcher::new().search_reader(sink.matcher.clone(), reader, sink) {
        Err(_) => Err(MError::Error),
        Ok(_) => Ok(()),
    };
}

// For files. Possibly faster.
fn search_path(sink: &mut StandardSink, path: &Path) -> Result<(), MError> {
    return match Searcher::new().search_path(sink.matcher.clone(), path, sink) {
        Err(_) => Err(MError::Error),
        Ok(_) => Ok(()),
    };
}

