use std::{
    fs::File,
    io::{self, BufRead, BufReader},
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

fn replace<R: BufRead>(opt: &Opt, mut bufr: R) -> Result<()> {
    let mut bs = Vec::<u8>::new();
    bufr.read_to_end(&mut bs)?;
    let haystack = String::from_utf8(bs)?;
    let replaced = opt.pattern.replace_all(&haystack, &opt.replacer);
    print!("{}", replaced);
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    debug!("{:?}", opt);
    let files = if opt.files.is_empty() {
        vec![PathBuf::from("-")]
    } else {
        opt.files.clone()
    };
    for file_path in files.iter() {
        if file_path.to_str() == Some("-") {
            let stdin = io::stdin();
            let stdin_lock = stdin.lock();
            let bufr = BufReader::new(stdin_lock);
            replace(&opt, bufr)?;
        } else {
            let file = File::open(file_path)?;
            let bufr = BufReader::new(file);
            replace(&opt, bufr)?;
        }
    }
    Ok(())
}
