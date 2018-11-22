
use std::error::Error;
use std::io;
use std::path::Path;

use grep::searcher::{Searcher, Sink, SinkMatch};

use standard::StandardSink;

pub fn search_path(sink: &mut StandardSink, path: &Path) -> Result<(), Box<Error>> {
    Searcher::new().search_path(sink.matcher.clone(), path, sink)?;
    Ok(())
}

