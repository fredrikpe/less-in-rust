use std::path::Path;

use grep::matcher::Match;
use grep::regex::RegexMatcher;
use grep::searcher::Searcher;

use error::MError;
use standard::StandardSink;

pub fn search(
    matches: &mut Vec<(u64, Match)>,
    input_file: &String,
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

    search_path(&mut sink, std::path::Path::new(input_file))?;

    Ok(())
}

fn search_path(sink: &mut StandardSink, path: &Path) -> Result<(), MError> {
    return match Searcher::new().search_path(sink.matcher.clone(), path, sink) {
        Err(_) => Err(MError::Error),
        Ok(_) => Ok(()),
    };
}
