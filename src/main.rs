use clap::{CommandFactory as _, Parser};
use clap_complete::CompleteEnv;
use jj_work::cli::{Cli, run};

fn main() -> miette::Result<()> {
    CompleteEnv::with_factory(Cli::command).complete();
    run(Cli::parse())
}
