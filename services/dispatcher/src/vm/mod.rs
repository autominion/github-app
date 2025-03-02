//! Access remote virtual machines.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use tokio_stream::StreamExt;

use config::Config;

mod aws;
mod local;

pub use aws::AwsVm;
pub use local::LocalVM;

#[async_trait]
pub trait VirtualMachine: Send {
    /// Create a new virtual machine.
    async fn create(config: &Config) -> Self
    where
        Self: Sized;

    /// Install Docker on the virtual machine.
    async fn install_docker(&mut self);

    /// Run bash code on the virtual machine.
    async fn run_command(&mut self, command: &str) -> CommandResult {
        println!("Running command: {}", command);

        let mut log_output = String::new();
        let mut exit_code = 0;

        let mut stream = self.run_command_stream(command).await;
        while let Some(log) = stream.next().await {
            match log {
                CommandOutput::StdoutLine(line) => {
                    log_output.push_str(&line);
                    log_output.push('\n');
                }
                CommandOutput::StderrLine(line) => {
                    log_output.push_str(&line);
                    log_output.push('\n');
                }
                CommandOutput::Exit(code) => {
                    exit_code = code;
                }
            }
        }

        println!("exit code: {}", exit_code);

        CommandResult { exit_code, log_output }
    }

    /// Run bash code on the virtual machine and stream the output.
    async fn run_command_stream(
        &mut self,
        code: &str,
    ) -> Pin<Box<dyn Stream<Item = CommandOutput> + Send>>;

    /// Detach from the virtual machine (e.g. close SSH connection).
    async fn detach(&mut self);

    /// Destroy the virtual machine.
    async fn destroy(self);
}

#[derive(Debug)]
pub enum CommandOutput {
    /// A line of stdout.
    StdoutLine(String),
    /// A line of stderr.
    StderrLine(String),
    /// Exit code of the command.
    Exit(i32),
}

#[allow(dead_code)]
pub struct CommandResult {
    pub exit_code: i32,
    pub log_output: String,
}
