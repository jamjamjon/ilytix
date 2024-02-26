use anyhow::Result;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::LOGGER;

pub fn make_folders<P: AsRef<Path>>(p: P) -> Result<PathBuf> {
    let p = p.as_ref();
    let mut saveout = p.to_path_buf();
    let name = match p.file_name() {
        None => panic!(
            "Error: Can not make folders becuase of the incorrect path: {:?}",
            p
        ),
        Some(name) => name.to_str().unwrap(),
    };
    let mut cnt = 1;
    while saveout.exists() {
        saveout.set_file_name(format!("{}-{}", name, cnt));
        cnt += 1;
    }
    std::fs::create_dir_all(&saveout)?;
    Ok(saveout)
}

pub fn build_pb(size: u64, prefix: &str) -> ProgressBar {
    let pb = ProgressBar::new(size);
    pb.set_style(
            ProgressStyle::with_template(
                "{prefix:.bold} [{bar:.blue.bright/white.dim}] {human_pos}/{human_len} ({percent}% | {eta} | {elapsed_precise})"
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write | write!(w, "{:.2}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
    pb.set_prefix(String::from("\n🐢 ") + prefix);
    pb
}

pub fn load_files<P: AsRef<Path>>(
    source: P,
    recursive: bool,
    hidden_include: bool,
) -> Result<Vec<PathBuf>> {
    let source = source.as_ref();
    if !source.exists() {
        LOGGER.exit("Source", " Not Exist", source.to_str().unwrap());
    }
    if source.is_symlink() {
        LOGGER.exit("Source", " Is Symlink", source.to_str().unwrap());
    }
    let ys = if source.is_file() {
        ("File", vec![source.to_path_buf()])
    } else {
        let mut ys: Vec<PathBuf> = Vec::new();
        for entry in WalkDir::new(source).into_iter().filter_entry(|x| {
            let x = x
                .file_name()
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false);
            if hidden_include {
                x
            } else {
                !x
            }
        }) {
            match entry {
                Ok(entry) => {
                    // non-recrusive
                    if !recursive && entry.depth() > 1 {
                        continue;
                    }
                    // symlink excluded
                    if entry.path_is_symlink() {
                        continue;
                    }
                    // directory excluded
                    if entry.file_type().is_dir() {
                        continue;
                    }
                    ys.push(entry.path().to_path_buf());
                }
                Err(_) => {
                    continue;
                }
            }
        }
        ("Folder", ys)
    };
    let source = source.canonicalize()?;
    LOGGER.success("Source", source.to_str().unwrap(), ys.0);
    LOGGER.success("Recursively", &format!("{:?}", &recursive), "");
    Ok(ys.1)
}

enum LoggerKind {
    Success,
    Warn,
    Fail,
}

pub struct Logger;
#[allow(clippy::println_empty_string)]
impl Logger {
    fn _log_text(&self, text: &str, style: console::Style) {
        print!("{}", style.apply_to(text));
    }

    fn _log_title(&self, kind: LoggerKind, text: &str) -> bool {
        if !text.is_empty() {
            match kind {
                LoggerKind::Success => {
                    self._log_text("✔  ", console::Style::new().bold().color256(49).bright());
                    self._log_text(text, console::Style::new().white().bold().bright());
                }
                LoggerKind::Fail => {
                    self._log_text("✘  ", console::Style::new().bold().color256(9).bright());
                    self._log_text(text, console::Style::new().white().bold().bright());
                }
                LoggerKind::Warn => {
                    self._log_text("✘  ", console::Style::new().bold().color256(220).bright());
                    self._log_text(text, console::Style::new().white().bold().bright());
                }
            }
        }
        text.is_empty()
    }

    fn _log_base(&self, kind: LoggerKind, t1: &str, t2: &str, prompt: &str) {
        let is_t1_empty = self._log_title(kind, t1);
        if !t2.is_empty() {
            self._log_text(
                &format!("{} · ", if is_t1_empty { "   " } else { "" }),
                console::Style::new().white().bright(),
            );
            self._log_text(t2, console::Style::new().color256(49).bright());
        }
        if !prompt.is_empty() {
            self._log_text(
                &format!("{} › ", if t2.is_empty() { "   " } else { "" }),
                console::Style::new().black().bright(),
            );
            self._log_text(prompt, console::Style::new().white().bright());
        }
        println!("");
    }

    pub fn success(&self, t1: &str, t2: &str, prompt: &str) {
        self._log_base(LoggerKind::Success, t1, t2, prompt);
    }
    pub fn warn(&self, t1: &str, t2: &str, prompt: &str) {
        self._log_base(LoggerKind::Warn, t1, t2, prompt);
    }
    pub fn exit(&self, t1: &str, t2: &str, prompt: &str) {
        self._log_base(LoggerKind::Fail, t1, t2, prompt);
        std::process::exit(0);
    }
}
