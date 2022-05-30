use clap::{ArgEnum, Parser};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(short, long, arg_enum)]
    pub task: CliTask,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum CliTask {
    UpdateSchools,
}
