use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;

use async_ssh2_lite::{AsyncSession, TokioTcpStream};
use async_trait::async_trait;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_sdk_ec2::operation::create_key_pair::CreateKeyPairOutput;
use aws_sdk_ec2::types::{
    BlockDeviceMapping, CreditSpecificationRequest, EbsBlockDevice, InstanceMarketOptionsRequest,
    InstanceStateName, InstanceType, KeyType, MarketType, SpotInstanceType, SpotMarketOptions,
    VolumeType,
};
use aws_types::region::Region;
use aws_types::SdkConfig;
use futures::future::FutureExt;
use futures::Stream;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use config::Config;
use tokio_stream::wrappers::ReceiverStream;

use super::{CommandOutput, VirtualMachine};

const SYSBOX_DEB_DOWNLOAD_URL: &str =
    "https://downloads.nestybox.com/sysbox/releases/v0.6.6/sysbox-ce_0.6.6-0.linux_amd64.deb";
const SYSBOX_DEB_SHA256: &str = "87cfa5cad97dc5dc1a243d6d88be1393be75b93a517dc1580ecd8a2801c2777a";

pub struct AwsVm {
    client: aws_sdk_ec2::Client,
    key_name: String,
    instance_id: String,
    ssh_session: AsyncSession<TokioTcpStream>,
    private_key: String,
}

#[async_trait]
impl VirtualMachine for AwsVm {
    async fn create(config: &Config) -> Self {
        // Create the AWS client
        let client = aws_sdk_ec2::Client::new(&sdk_config(config));

        // Generate a key name (up to 255 ASCII characters)
        let key_name = crate::tokens::alphanumeric("minion-", 128);

        // Generate a key pair
        let key_pair_res = generate_key_pair(&client, &key_name).await;
        let private_key = key_pair_res.key_material().unwrap();

        println!("{}", private_key);

        // Start the AWS instance
        let instance_id = run_instance(&client, &config.aws_image_id, &key_name).await;

        println!("Waiting for instance to be ready ...");

        wait_for_instance_running(&client, &instance_id).await;

        println!("instance ready: {}", instance_id);

        let ip_address = get_instance_ip_address(&client, &instance_id).await;

        println!("ip address: {}", ip_address);

        println!("Connecting via SSH ...");

        let ssh_session = create_ssh_connection(&ip_address, private_key).await;

        Self {
            client,
            key_name,
            instance_id: instance_id.to_owned(),
            ssh_session,
            private_key: private_key.to_owned(),
        }
    }

    async fn install_docker(&mut self) {
        self.run_command("sudo apt-get update").await;
        self.run_command("sudo apt-get install -y docker.io").await;
        self.run_command("sudo usermod -aG docker ubuntu").await;
        // Reconnect to SSH s.t. adding the docker group takes effect
        self.ssh_session.disconnect(None, "", None).await.unwrap();
        self.ssh_session = create_ssh_connection(
            &get_instance_ip_address(&self.client, &self.instance_id).await,
            &self.private_key,
        )
        .await;
        self.install_sysbox().await;
    }

    async fn run_command_stream(
        &mut self,
        command: &str,
    ) -> Pin<Box<dyn Stream<Item = CommandOutput> + Send>> {
        println!("Running command on AWS instance: {}", command);

        let mut channel =
            self.ssh_session.channel_session().await.expect("Failed to create channel session");
        channel.exec(command).await.expect("Failed to execute command");

        let stdout_stream = channel.stream(0);
        let stderr_stream = channel.stderr();

        let stdout_reader = BufReader::new(stdout_stream);
        let stderr_reader = BufReader::new(stderr_stream);

        let (tx, rx) = mpsc::channel(32);

        // Stream stdout.
        let tx_stdout = tx.clone();
        tokio::spawn(async move {
            let mut lines = stdout_reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_stdout.send(CommandOutput::StdoutLine(line)).await.is_err() {
                    break;
                }
            }
        });

        // Stream stderr.
        let tx_stderr = tx.clone();
        tokio::spawn(async move {
            let mut lines = stderr_reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_stderr.send(CommandOutput::StderrLine(line)).await.is_err() {
                    break;
                }
            }
        });

        // Wait for the command to complete.
        let tx_exit = tx;
        tokio::spawn(async move {
            if let Err(e) = channel.wait_eof().await {
                eprintln!("Error waiting for channel EOF: {:?}", e);
            }
            if let Err(e) = channel.wait_close().await {
                eprintln!("Error waiting for channel close: {:?}", e);
            }
            let exit_code = channel.exit_status().unwrap_or(-1);
            let _ = tx_exit.send(CommandOutput::Exit(exit_code)).await;
        });

        Box::pin(ReceiverStream::new(rx))
    }

    async fn detach(&mut self) {
        self.ssh_session.disconnect(None, "", None).await.unwrap();
    }

    async fn destroy(self) {
        self.client.delete_key_pair().key_name(self.key_name).send().await.unwrap();
        self.client.terminate_instances().instance_ids(self.instance_id).send().await.unwrap();
    }
}

impl AwsVm {
    async fn install_sysbox(&mut self) {
        self.run_command("sudo apt-get update && sudo apt-get install -y wget jq").await;

        let tmp_dir_output = self.run_command("mktemp -d").await;
        let tmp_dir = tmp_dir_output.log_output.trim();
        println!("Temporary directory created: {}", tmp_dir);

        let deb_path = format!("{}/sysbox.deb", tmp_dir);
        self.run_command(&format!("wget -O {} {}", deb_path, SYSBOX_DEB_DOWNLOAD_URL)).await;

        let checksum_output = self.run_command(&format!("sha256sum {}", deb_path)).await;
        let actual_checksum = checksum_output.log_output.split_whitespace().next().unwrap_or("");

        if actual_checksum == SYSBOX_DEB_SHA256 {
            println!("Checksum verified successfully.");
            self.run_command(&format!("sudo dpkg -i {}", deb_path)).await;
        } else {
            eprintln!("Checksum verification failed!");
            eprintln!("Expected: {}", SYSBOX_DEB_SHA256);
            eprintln!("Actual: {}", actual_checksum);
            panic!("Aborting installation due to checksum mismatch.");
        }

        self.run_command(&format!("rm -rf {}", tmp_dir)).await;
    }
}

fn sdk_config(config: &Config) -> SdkConfig {
    SdkConfig::builder()
        .region(Some(Region::new(config.aws_region.clone())))
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            &config.aws_access_key_id,
            &config.aws_secret_access_key,
            None,
            None,
            "minion",
        )))
        .build()
}

async fn generate_key_pair(client: &aws_sdk_ec2::Client, key_name: &str) -> CreateKeyPairOutput {
    client.create_key_pair().key_name(key_name).key_type(KeyType::Ed25519).send().await.unwrap()
}

async fn run_instance(client: &aws_sdk_ec2::Client, aws_image_id: &str, key_name: &str) -> String {
    let res = client
        .run_instances()
        .image_id(aws_image_id)
        .min_count(1)
        .max_count(1)
        .key_name(key_name)
        // In AWS for burstable performance instances (e.g., T2, T3, T4g), setting the CPU credit
        // specification to "standard" means the instance is limited by its accrued CPU credits.
        // Once it runs out of credits, its CPU performance is throttled until it regenerates more credits.
        // This helps control costs by preventing unexpected over-usage without additional charges.
        .credit_specification(CreditSpecificationRequest::builder().cpu_credits("standard").build())
        .instance_market_options(
            InstanceMarketOptionsRequest::builder()
                .market_type(MarketType::Spot)
                .spot_options(
                    SpotMarketOptions::builder()
                        .spot_instance_type(SpotInstanceType::OneTime)
                        .build(),
                )
                .build(),
        )
        .block_device_mappings(
            BlockDeviceMapping::builder()
                .device_name("/dev/sda1")
                // 0.0001304$/GB/h, 32GB => 0.00417$/h
                .ebs(EbsBlockDevice::builder().volume_size(32).volume_type(VolumeType::Gp3).build())
                .build(),
        )
        // 0.4288$/h, 8vCPU, 32GB RAM
        .instance_type(InstanceType::T22xlarge)
        .security_groups("minion-dev")
        .send()
        .await
        .unwrap();

    let instance = res.instances().unwrap().first().unwrap();
    instance.instance_id().unwrap().to_owned()
}

async fn wait_for_instance_running(client: &aws_sdk_ec2::Client, instance_id: &str) {
    wait_for_bool(|| is_instance_running(client, instance_id)).await
}

/// Check if an instance is running.
async fn is_instance_running(client: &aws_sdk_ec2::Client, instance_id: &str) -> bool {
    let res = client
        .describe_instance_status()
        .instance_ids(instance_id)
        .include_all_instances(true)
        .send()
        .await
        .unwrap();

    let status = res.instance_statuses().unwrap().first().unwrap();
    let state = status.instance_state().unwrap().name().unwrap();

    matches!(state, InstanceStateName::Running)
}

async fn get_instance_ip_address(client: &aws_sdk_ec2::Client, instance_id: &str) -> String {
    let res = client.describe_instances().instance_ids(instance_id).send().await.unwrap();

    let ip_address = res.reservations().unwrap()[0]
        .instances()
        .unwrap()
        .first()
        .unwrap()
        .public_ip_address()
        .unwrap();

    ip_address.to_owned()
}

async fn create_ssh_connection(
    ip_address: &str,
    private_key: &str,
) -> AsyncSession<TokioTcpStream> {
    let ip_addr: IpAddr = ip_address.parse().unwrap();
    let ssh_address = SocketAddr::from((ip_addr, 22));
    let mut ssh_session = wait_connect_ssh(ssh_address).await;
    ssh_session.handshake().await.unwrap();
    ssh_session.userauth_pubkey_memory("ubuntu", None, private_key, None).await.unwrap();
    ssh_session
}

async fn wait_connect_ssh(address: SocketAddr) -> AsyncSession<TokioTcpStream> {
    wait_for(|| connect_ssh(address)).await
}

async fn connect_ssh(address: SocketAddr) -> Option<AsyncSession<TokioTcpStream>> {
    let res = AsyncSession::<TokioTcpStream>::connect(address, None).await;
    match res {
        Ok(session) => Some(session),
        Err(err) => {
            eprintln!("SSH connection error: {:?}", err);
            None
        }
    }
}

async fn wait_for_bool<OF, F: Fn() -> OF>(f: F)
where
    OF: Future<Output = bool>,
{
    wait_for(|| f().map(|out| out.then_some(()))).await
}

async fn wait_for<O, OF, F: Fn() -> OF>(f: F) -> O
where
    OF: Future<Output = Option<O>>,
{
    loop {
        let res = time::sleep(Duration::from_secs(5)).then(|_| f()).await;
        if let Some(output) = res {
            return output;
        }
    }
}
