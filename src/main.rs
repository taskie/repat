use std::path::PathBuf;

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
    Ok(())
}
