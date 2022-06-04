use clap::{ArgEnum, Parser};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(short, long, arg_enum, required = true, min_values = 1)]
    pub task: Vec<CliTask>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum CliTask {
    UpdateSchools,
    UpdateTube,
}
