use clap::{App as ClapApp, Arg, ArgMatches};

pub struct App {
    pub matches: ArgMatches<'static>,
}

impl App {
    pub fn new() -> App {
        App {
            matches: clap_app().get_matches(),
        }
    }

    fn files(&self) -> Vec<InputFile> {
        self.matches
            .values_of("FILE")
            .map(|values| {
                values
                    .map(|filename| {
                        if filename == "-" {
                            InputFile::StdIn
                        } else {
                            InputFile::Ordinary(filename)
                        }
                    })
                .collect()
            })
        .unwrap_or_else(|| vec![InputFile::StdIn])
    }
}

fn clap_app() -> ClapApp<'static, 'static> {
    ClapApp::new("less2")
        .version("0.0.1")
        .about(
            "About",
            )
        .long_about("Long about")
        .arg(
            Arg::with_name("FILE")
            .help("File(s) to print / concatenate. Use '-' for standard input.")
            .long_help(
                "File(s) to print / concatenate. Use a dash ('-') or no \
                        argument at all to read from standard input.",
                        )
            .multiple(true)
            .empty_values(false),
            )
        .help_message("Print this help message.")
        .version_message("Show version information.")
}


