use ansi_term::Colour;
use anyhow::Result;
use regex::Regex;
use similar::TextDiff;
use std::{
    collections::{btree_map::Entry, BTreeMap},
    env,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "repat", about = "Regex pattern viewer")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    #[structopt(name = "FIND", help = "Regex.")]
    find: String,

    #[structopt(name = "REPLACE_WITH", help = "Replacer.")]
    replace_with: String,

    #[structopt(name = "FILE", help = "Input files.")]
    files: Vec<PathBuf>,

    #[structopt(short, long, help = "Dumps result as Unified Format diff/patch file.")]
    diff: bool,
}

fn process_text<R: BufRead, W: Write>(
    _opt: &Opt,
    r: &mut R,
    w: &mut W,
    re: &Regex,
    replacer: &str,
) -> Result<()> {
    let colors = [
        Colour::Red,
        Colour::Green,
        Colour::Yellow,
        Colour::Blue,
        Colour::Purple,
        Colour::Cyan,
    ];
    for line in r.lines() {
        let line = line?;
        let bs = line.as_bytes();
        // TODO: change this to smarter data structure...
        let mut segment = vec![usize::MAX; bs.len()];
        let mut expanded_map = BTreeMap::<usize, Vec<String>>::new();
        for captures in re.captures_iter(&line) {
            for (i, capture) in captures.iter().enumerate() {
                let capture = match capture {
                    Some(m) => m,
                    None => continue,
                };
                let start = capture.start();
                let end = capture.end();
                for j in start..end {
                    segment[j] = i;
                }
                if i == 0 {
                    let mut expanded = String::new();
                    captures.expand(replacer, &mut expanded);
                    match expanded_map.entry(end) {
                        Entry::Vacant(v) => {
                            v.insert(vec![expanded]);
                        }
                        Entry::Occupied(mut o) => o.get_mut().push(expanded),
                    };
                }
            }
        }
        let mut pos = 0usize;
        let mut last_status = usize::MAX;
        let print = |w: &mut dyn Write, start, end, status| -> Result<()> {
            let color: Option<Colour> = if status == usize::MAX {
                None
            } else {
                Some(colors[status % colors.len()])
            };
            let s = String::from_utf8(bs[start..end].to_vec())?;
            match color {
                Some(c) => write!(w, "{}", c.bold().strikethrough().paint(s))?,
                None => write!(w, "{}", s)?,
            };
            Ok(())
        };
        let expand = move |w: &mut dyn Write, idx: usize| -> Result<()> {
            for expanded_list in expanded_map.get(&idx) {
                for expanded in expanded_list {
                    write!(w, "{}", Colour::White.bold().underline().paint(expanded))?;
                }
            }
            Ok(())
        };
        for (idx, status) in segment.into_iter().enumerate() {
            if status != last_status {
                print(w, pos, idx, last_status)?;
                pos = idx;
                last_status = status;
            }
            expand(w, idx-1)?;
        }
        print(w, pos, bs.len(), last_status)?;
        expand(w, bs.len()-1)?;
        expand(w, bs.len())?;
        writeln!(w)?;
    }
    Ok(())
}

fn process_diff<R: BufRead, W: Write>(
    _opt: &Opt,
    r: &mut R,
    w: &mut W,
    from: &str,
    to: &str,
    re: &Regex,
    replacer: &str,
) -> Result<()> {
    let mut before: Vec<u8> = Vec::new();
    r.read_to_end(&mut before)?;
    let before = String::from_utf8(before)?;
    let after = re.replace_all(&before, replacer);
    let after: String = after.into();
    let diff = TextDiff::from_lines(&before, &after);
    diff.unified_diff().header(from, to).to_writer(w)?;
    Ok(())
}

fn process_read<R: BufRead, W: Write>(
    opt: &Opt,
    r: &mut R,
    w: &mut W,
    path: &str,
    re: &Regex,
    replacer: &str,
) -> Result<()> {
    if opt.diff {
        process_diff(opt, r, w, path, path, re, replacer)
    } else {
        process_text(opt, r, w, re, replacer)
    }
}

fn process_file<W: Write>(
    opt: &Opt,
    w: &mut W,
    path: &Path,
    re: &Regex,
    replacer: &str,
) -> Result<()> {
    let file = File::open(path)?;
    let mut buf = BufReader::new(file);
    process_read(&opt, &mut buf, w, path.to_str().unwrap(), &re, replacer)
}

fn process_stdin<W: Write>(opt: &Opt, w: &mut W, re: &Regex, replacer: &str) -> Result<()> {
    let stdin = stdin();
    let stdin = stdin.lock();
    let mut buf = BufReader::new(stdin);
    process_read(&opt, &mut buf, w, "-", &re, replacer)
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    #[cfg(windows)]
    let enabled = ansi_term::enable_ansi_support();
    let re = Regex::new(&opt.find)?;
    let replacer = opt.replace_with.clone();
    let stdout = stdout();
    let stdout = stdout.lock();
    let mut out = BufWriter::new(stdout);
    let mut files = opt.files.clone();
    if files.is_empty() {
        files.push("-".into());
    }
    for file in files {
        if file.to_str() == Some("-") {
            process_stdin(&opt, &mut out, &re, &replacer)?;
        } else {
            process_file(&opt, &mut out, &file, &re, &replacer)?;
        }
    }
    Ok(())
}
