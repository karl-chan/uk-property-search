mod cli;
mod lib;
mod tasks;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, CliTask};
use lib::util::globals::Globals;
use tasks::update_schools::update_schools;

#[tokio::main]
async fn main() -> Result<()> {
    let globals = Globals::new().await;

    let args = Cli::parse();
    match args.task {
        CliTask::UpdateSchools => update_schools(&globals).await,
    }
}
