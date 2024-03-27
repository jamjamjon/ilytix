#[derive(clap::Subcommand, Debug)]
pub enum Task {
    /// Images integrity checking
    Check(crate::sanitizer::Args),

    /// Images de-duplicating
    Dedup(crate::deduplicator::Args),

    /// Image-Image supported. TODO: Text-Image
    Retrive(crate::retrival::Args),

    /// TODO
    Caption,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub task: Task,
}
