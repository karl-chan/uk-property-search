mod cli;
mod lib;
mod tasks;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, CliTask};
use lib::util::globals::Globals;
use log::info;
use stopwatch::Stopwatch;
use tasks::{
    update_property::update_property, update_schools::update_schools, update_tube::update_tube,
};

#[tokio::main]
async fn main() -> Result<()> {
    let globals = Globals::new().await;

    let args = Cli::parse();
    for task in args.task {
        let sw = Stopwatch::start_new();
        match task {
            CliTask::UpdateProperty => update_property(&globals).await?,
            CliTask::UpdateSchools => update_schools(&globals).await?,
            CliTask::UpdateTube => update_tube(&globals).await?,
        }
        info!("Completed task [{:?}] in [{:?}].", task, sw.elapsed());
    }
    Ok(())
}
