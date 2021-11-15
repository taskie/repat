use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use anyhow::Result;
use log::debug;
use regex::Regex;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "repat", about = "Creates patch to replace words using RegEx.")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
struct Opt {
    #[structopt(name = "PATTERN")]
    pattern: Regex,

    #[structopt(name = "REPLACER")]
    replacer: String,

    #[structopt(name = "FILE")]
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    debug!("{:?}", opt);
    for file_path in opt.files.iter() {
        let file = File::open(file_path)?;
        let mut bufr = BufReader::new(file);
        let mut bs = Vec::<u8>::new();
        bufr.read_to_end(&mut bs)?;
        let haystack = String::from_utf8(bs)?;
        let replaced = opt.pattern.replace_all(&haystack, &opt.replacer);
        print!("{}", replaced);
    }
    Ok(())
}
