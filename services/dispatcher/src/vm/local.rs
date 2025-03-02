use std::pin::Pin;

use async_trait::async_trait;

use config::Config;
use futures::Stream;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use super::{CommandOutput, VirtualMachine};

pub struct LocalVM {}

#[async_trait]
impl VirtualMachine for LocalVM {
    async fn create(_config: &Config) -> Self {
        Self {}
    }

    async fn install_docker(&mut self) {}

    async fn run_command_stream(
        &mut self,
        code: &str,
    ) -> Pin<Box<dyn Stream<Item = CommandOutput> + Send>> {
        use std::process::Stdio;

        let mut child = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(code)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn command");

        let stdout = child.stdout.take().expect("failed to capture stdout");
        let stderr = child.stderr.take().expect("failed to capture stderr");

        let (tx, rx) = mpsc::channel(32);

        // Stream stdout.
        let tx_stdout = tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_stdout.send(CommandOutput::StdoutLine(line)).await.is_err() {
                    break;
                }
            }
        });

        // Stream stderr.
        let tx_stderr = tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_stderr.send(CommandOutput::StderrLine(line)).await.is_err() {
                    break;
                }
            }
        });

        // Wait for the command to complete.
        tokio::spawn(async move {
            let status = child.wait().await.expect("failed to wait on child");
            let exit_code = status.code().unwrap_or(-1);
            let _ = tx.send(CommandOutput::Exit(exit_code)).await;
        });

        Box::pin(ReceiverStream::new(rx))
    }

    async fn detach(&mut self) {}

    async fn destroy(self) {}
}
