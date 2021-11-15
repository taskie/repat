use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::{exit, Command},
};

use anyhow::{Context, Result};
use log::debug;
use regex::Regex;
use similar::TextDiff;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "repat", about = "Creates patch to replace words using RegEx.")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
struct Opt {
    #[structopt(short, long)]
    rg: bool,

    #[structopt(short = "s", long)]
    searcher: Option<String>,

    #[structopt(short = "S", long)]
    searcher_flags: Option<String>,

    #[structopt(short = "0", long)]
    null: Option<bool>,

    #[structopt(name = "PATTERN")]
    pattern: Regex,

    #[structopt(name = "REPLACER")]
    replacer: String,

    #[structopt(name = "FILE")]
    files: Vec<PathBuf>,
}

fn exec_searcher(
    opt: &Opt,
    searcher: &str,
    searcher_flags: &[String],
    null: bool,
) -> Result<Vec<PathBuf>> {
    let command = Command::new(&searcher)
        .args(searcher_flags)
        .arg(opt.pattern.as_str())
        .output()?;
    debug!("{:?}", command);
    if !command.status.success() {
        exit(1);
    }
    let delimiter = if null { '\0' } else { '\n' };
    let files = command.stdout.split(|b| *b as char == delimiter);
    let files: Result<Vec<String>> = files
        .map(|bs| String::from_utf8(bs.to_vec()).context("not UTF-8"))
        .collect();
    Ok(files?
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.into())
        .collect())
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
    let mut files = opt.files.clone();
    if opt.rg || opt.searcher.is_some() {
        let searcher = opt.searcher.clone().unwrap_or_else(|| "rg".to_string());
        let searcher_flags = if let Some(flags) = opt.searcher_flags.clone() {
            flags.split(' ').map(|s| s.to_owned()).collect()
        } else {
            vec!["-0l".to_owned()]
        };
        let null = opt.null.unwrap_or(true);
        files.extend(exec_searcher(&opt, &searcher, &searcher_flags, null)?);
    };
    if files.is_empty() {
        files.push(PathBuf::from("-"));
    }
    let stdout = io::stdout();
    let stdout_lock = stdout.lock();
    let mut bufw = BufWriter::new(stdout_lock);
    for file_path in files.iter() {
        debug!("{:?}", file_path);
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
