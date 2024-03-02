use crate::{DeDuplicator, Retrival, Sanitizer};

#[derive(clap::Subcommand, Debug)]
pub enum Task {
    /// Images integrity checking
    Check(Sanitizer),

    /// Images de-duplicating
    Dedup(DeDuplicator),

    /// Image-Image supported. TODO: Text-Image
    Retrive(Retrival),

    /// TODO
    Caption,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub task: Task,
}
