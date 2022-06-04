mod cli;
mod lib;
mod tasks;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, CliTask};
use lib::util::globals::Globals;
use tasks::{update_schools::update_schools, update_tube::update_tube};

#[tokio::main]
async fn main() -> Result<()> {
    let globals = Globals::new().await;

    let args = Cli::parse();
    for task in args.task {
        match task {
            CliTask::UpdateSchools => update_schools(&globals).await?,
            CliTask::UpdateTube => update_tube(&globals).await?,
        }
    }
    Ok(())
}
