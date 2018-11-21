
use std::io::{self};

use grep::matcher::{Match, Matcher};
use grep::regex::RegexMatcher;
use grep::searcher::{Searcher, Sink, SinkMatch};

#[derive(Debug)]
pub struct StandardSink<'a> {
    pub matcher: RegexMatcher,
    pub matches: &'a mut Vec<(u64, Match)>,
    pub match_count: u64,
}

impl<'a> StandardSink<'a> {
    fn record_match(&mut self, mat: &SinkMatch) -> io::Result<()> {
        let bytes = mat.bytes();
        let offset = mat.absolute_byte_offset();

        let matches = &mut self.matches;
        self.matcher
            .find_iter(bytes, |m| {
                matches.push((offset, m));
                true
            })?;
            //.map_err(io::Error::error_message)?;
        // Don't report empty matches appearing at the end of the bytes.
        if !matches.is_empty()
            && matches.last().unwrap().1.is_empty()
            && matches.last().unwrap().1.start() >= bytes.len()
        {
            matches.pop().unwrap();
        }
        Ok(())
    }
}

impl<'a> Sink for StandardSink<'a> {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch) -> Result<bool, io::Error> {
        self.match_count += 1;
        self.record_match(mat)?;

        Ok(true)
    }
}
