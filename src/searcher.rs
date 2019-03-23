use std::path::Path;

use grep::matcher::Match;
use grep::regex::RegexMatcher;
use grep::searcher::Searcher;

use error::MError;
use standard::StandardSink;

/*
/// Search the given subject using the appropriate strategy.
fn search(&mut self, subject: &Subject) -> io::Result<SearchResult> {
    let path = subject.path();
    if subject.is_stdin() {
        let stdin = io::stdin();
        // A `return` here appeases the borrow checker. NLL will fix this.
        return self.search_reader(path, stdin.lock());
    } else if self.should_preprocess(path) {
        self.search_preprocessor(path)
    } else if self.should_decompress(path) {
        self.search_decompress(path)
    } else {
        self.search_path(path)
    }
}
*/

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
