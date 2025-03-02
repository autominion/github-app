use std::future::Future;

use clap::{Parser, Subcommand};

use crate::daemon;

pub fn exec() {
    let cli = Cli::parse();
    match cli.command {
        Command::Daemon(args) => run_async(daemon::exec(args)),
    }
}

#[derive(Parser)]
#[clap()]
struct Cli {
    #[clap(subcommand)]
    command: Command,
    /// Enable internal debug output
    #[clap(long, num_args = 0)]
    trace: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Start the daemon
    Daemon(daemon::Args),
}

fn run_async<F: Future<Output = ()>>(f: F) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(f)
}
