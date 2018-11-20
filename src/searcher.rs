use std::env;
use std::error::Error;
use std::io;
use std::path::Path;
use std::process;

use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{Sink, Searcher, SinkMatch};

pub fn search_path(pattern: &str, sink: &mut OffsetSink, path: &Path) -> Result<(), Box<Error>> {
    let matcher = RegexMatcher::new(pattern)?;
    eprintln!("pattern = {}", pattern);
    Searcher::new().search_path(
        &matcher,
        path,
        sink
        //UTF8(|lnum, line| {
            //eprint!("{}:{}", lnum, line);
            //Ok(true)
        //}),
    )?;
    Ok(())
}


#[derive(Clone, Debug)]
pub struct OffsetSink {
    pub offsets: Vec<u64>,
    pub lengths: Vec<usize>,
}

impl Sink for OffsetSink {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch) -> Result<bool, io::Error> {
        self.offsets.push(mat.absolute_byte_offset());
        self.lengths.push(mat.bytes().len());
        eprintln!("Found offset at {}, len = {}", mat.absolute_byte_offset(), mat.bytes().len());
        eprintln!("String is {}", std::str::from_utf8(mat.bytes()).unwrap());
        
        Ok(true)
    }
}
