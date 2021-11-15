use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::debug;
use regex::Regex;
use similar::TextDiff;
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

fn replace<R: BufRead, W: Write>(opt: &Opt, mut bufr: R, w: W, file_path: &Path) -> Result<()> {
    let mut bs = Vec::<u8>::new();
    bufr.read_to_end(&mut bs)?;
    let haystack = String::from_utf8(bs)?;
    let replaced = opt.pattern.replace_all(&haystack, &opt.replacer);
    let diff = TextDiff::from_lines(haystack.as_str(), &replaced);
    let path_str = file_path.to_string_lossy();
    diff.unified_diff()
        .header(&path_str, &path_str)
        .to_writer(w)?;
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
    let stdout = io::stdout();
    let stdout_lock = stdout.lock();
    let mut bufw = BufWriter::new(stdout_lock);
    for file_path in files.iter() {
        if file_path.to_str() == Some("-") {
            let stdin = io::stdin();
            let stdin_lock = stdin.lock();
            let bufr = BufReader::new(stdin_lock);
            replace(&opt, bufr, &mut bufw, file_path)?;
        } else {
            let file = File::open(file_path)?;
            let bufr = BufReader::new(file);
            replace(&opt, bufr, &mut bufw, file_path)?;
        }
    }
    Ok(())
}
