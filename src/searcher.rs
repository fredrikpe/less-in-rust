
use std::error::Error;
use std::io;
use std::path::Path;

use grep::searcher::{Searcher, Sink, SinkMatch};

use standard::StandardSink;

pub fn search_path(sink: &mut StandardSink, path: &Path) -> Result<(), Box<Error>> {
    Searcher::new().search_path(sink.matcher.clone(), path, sink)?;
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
        eprintln!(
            "Found offset at {}, len = {}",
            mat.absolute_byte_offset(),
            mat.bytes().len()
        );
        eprintln!("String is {}", std::str::from_utf8(mat.bytes()).unwrap());

        Ok(true)
    }
}
