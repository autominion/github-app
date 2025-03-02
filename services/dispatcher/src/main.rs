//! The dispatcher watches for new jobs and starts VMs to run them.

mod cli;
mod daemon;
mod tokens;
mod vm;

fn main() {
    cli::exec();
}
