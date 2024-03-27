use anyhow::Result;
use clap::Parser;

use ilytix::{Cli, Task};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.task {
        Task::Check(x) => {
            x.run()?;
        }
        Task::Dedup(x) => {
            x.run()?;
        }
        Task::Retrive(x) => {
            x.run()?;
        }
        _ => {
            todo!()
        }
    }
    Ok(())
}
