use clap::{CommandFactory as _, Parser};
use clap_complete::CompleteEnv;
use jj_work::cli::{Cli, run};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    CompleteEnv::with_factory(Cli::command).complete();
    run(Cli::parse())
}
