use crate::{DeDuplicator, Sanitizer};

#[derive(clap::Subcommand, Debug)]
pub enum Task {
    /// Images integrity checking
    Check(Sanitizer),

    /// Images de-duplicating
    Dedup(DeDuplicator),

    /// TODO
    Caption,

    /// TODO
    Retrival,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub task: Task,
}
