use std::fs;
use std::path::{Path, PathBuf};

use pkcs8::der::asn1::OctetString;
use pkcs8::der::Decode;
use pkcs8::{ObjectIdentifier, PrivateKeyInfo, SubjectPublicKeyInfo};
use serde::Deserialize;
use url::Url;

const ED25519_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.101.112");

/// The main configuration
#[derive(Clone)]
pub struct Config {
    /// The user-facing name of the service
    pub service_name: String,
    pub host: String,
    pub port: u16,
    pub openai_key: String,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_region: String,
    pub aws_image_id: String,
    pub github_git_name: String,
    pub github_git_email: String,
    pub github_app_id: String,
    pub github_app_client_id: String,
    pub github_app_client_secret: String,
    /// GitHub app webhook secret
    pub github_app_webhook_secret: String,
    /// Contents of the GitHub app private key
    pub github_app_private_key: String,
    /// ID of the GitHub app installation on which the GitHub app is installed
    /// Currently, only a single installation is supported
    pub github_app_installation_id: i64,
    /// ID of the GitHub organization on which the GitHub app is installed
    pub github_app_organization_id: String,
    /// Name of the GitHub organization on which the GitHub app is installed
    pub github_app_organization_name: String,
    /// The handle of the GitHub bot including the leading '@', e.g. "@handle"
    pub github_bot_handle: String,
    /// The User-Agent string to use when making requests to the GitHub API
    pub github_api_user_agent: String,
    pub postgres_url: Url,
    pub web_base_url: Url,
    // Default agent configuration
    pub default_agent_container_registry_host: String,
    pub default_agent_container_registry_username: String,
    pub default_agent_container_registry_password: String,
    pub default_agent_container_image: String,
    // JWT secret keys
    pub jwt_expanded_private_key: Vec<u8>,
    pub jwt_public_key: Vec<u8>,
    /// Access control mode
    pub access_control: AccessControl,
    /// Email addresses that are whitelisted
    pub whitelisted_emails: Vec<String>,
    /// Whether to dispatch jobs
    pub dispatch_mode: DispatchMode,
    pub s3_endpoint: Url,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_prefix: String,
    /// Directory of static files to serve
    pub static_dir: PathBuf,
}

impl Config {
    /// Load the config from `config.toml`
    pub fn load() -> Self {
        let env_vars: ConfigEnvVars =
            envy::prefixed("MINION_").from_env::<ConfigEnvVars>().unwrap();
        let config_file_path = env_vars.config_file.unwrap_or("config.toml".into());

        let text = fs::read_to_string(config_file_path).unwrap();
        let file: ConfigFile = toml::from_str(&text).unwrap();

        let github_app_private_key = fs::read_to_string(file.github_app_private_key).unwrap();

        let (jwt_public_key, jwt_expanded_private_key) =
            load_jwt_keys(&file.jwt_private_key, &file.jwt_public_key);

        Config {
            service_name: file.service_name,
            host: file.host,
            port: file.port,
            openai_key: file.openai_key,
            aws_access_key_id: file.aws_access_key_id,
            aws_secret_access_key: file.aws_secret_access_key,
            aws_region: file.aws_region,
            aws_image_id: file.aws_image_id,
            github_app_id: file.github_app_id,
            github_git_name: file.github_git_name,
            github_git_email: file.github_git_email,
            github_app_client_id: file.github_app_client_id,
            github_app_client_secret: file.github_app_client_secret,
            github_app_webhook_secret: file.github_app_webhook_secret,
            github_app_private_key,
            github_app_installation_id: file.github_app_installation_id,
            github_app_organization_id: file.github_app_organization_id,
            github_app_organization_name: file.github_app_organization_name,
            github_bot_handle: file.github_bot_handle,
            github_api_user_agent: file.github_api_user_agent,
            postgres_url: file.postgres_url,
            web_base_url: env_vars.web_base_url.unwrap_or(file.web_base_url),
            default_agent_container_registry_host: file.default_agent_container_registry_host,
            default_agent_container_registry_username: file
                .default_agent_container_registry_username,
            default_agent_container_registry_password: file
                .default_agent_container_registry_password,
            default_agent_container_image: file.default_agent_container_image,
            jwt_expanded_private_key,
            jwt_public_key,
            access_control: file.access_control,
            whitelisted_emails: file.allowed_emails.unwrap_or_default(),
            dispatch_mode: file.dispatch_mode.unwrap_or_default(),
            s3_endpoint: file.s3_endpoint,
            s3_region: file.s3_region,
            s3_bucket: file.s3_bucket,
            s3_access_key: file.s3_access_key,
            s3_secret_key: file.s3_secret_key,
            s3_prefix: file.s3_prefix,
            static_dir: file.static_dir,
        }
    }
}

fn load_jwt_keys(private_key_path: &Path, public_key_path: &Path) -> (Vec<u8>, Vec<u8>) {
    // Read the private key PEM file
    let private_pem = fs::read_to_string(private_key_path).unwrap_or_else(|_| {
        panic!("Failed to read private key file: {}", private_key_path.display())
    });

    // Parse the PEM to extract DER-encoded contents
    let private_pem = pem::parse(&private_pem).expect("Failed to parse private key PEM");

    // Parse the PKCS#8 structure to access the private_key field
    let pkcs8 = PrivateKeyInfo::from_der(&private_pem.contents)
        .expect("Failed to parse PKCS#8 private key");

    // Verify that the private key algorithm is Ed25519
    if pkcs8.algorithm.oid != ED25519_OID {
        panic!(
            "Unexpected private key algorithm OID: expected {}, got {}",
            ED25519_OID, pkcs8.algorithm.oid
        );
    }

    // Extract the inner OCTET STRING containing the raw 32-byte private key
    let jwt_private_key = OctetString::from_der(pkcs8.private_key)
        .expect("Failed to parse inner private key OCTET STRING")
        .as_bytes()
        .to_vec();

    // Ensure the private key is exactly 32 bytes
    assert_eq!(
        jwt_private_key.len(),
        32,
        "Invalid private key length: expected 32 bytes, got {} bytes",
        jwt_private_key.len()
    );

    // Read the public key PEM file
    let public_pem = fs::read_to_string(public_key_path).unwrap_or_else(|_| {
        panic!("Failed to read public key file: {}", public_key_path.display())
    });

    // Parse the PEM to extract DER-encoded contents
    let public_pem = pem::parse(&public_pem).expect("Failed to parse public key PEM");

    // Parse the SubjectPublicKeyInfo structure to access the subject_public_key field
    let spki = SubjectPublicKeyInfo::from_der(&public_pem.contents)
        .expect("Failed to parse SubjectPublicKeyInfo");

    // Verify that the public key algorithm is Ed25519
    if spki.algorithm.oid != ED25519_OID {
        panic!(
            "Unexpected public key algorithm OID: expected {}, got {}",
            ED25519_OID, spki.algorithm.oid
        );
    }

    // Extract the raw public key bytes using as_bytes()
    let jwt_public_key = spki.subject_public_key.to_vec();

    // Ensure the public key is exactly 32 bytes
    assert_eq!(
        jwt_public_key.len(),
        32,
        "Invalid public key length: expected 32 bytes, got {} bytes",
        jwt_public_key.len()
    );

    // Concatenate the 32-byte private key and 32-byte public key to form a 64-byte expanded private key
    let jwt_expanded_private_key = [jwt_private_key.clone(), jwt_public_key.clone()].concat();

    // Ensure the expanded private key is exactly 64 bytes
    assert_eq!(
        jwt_expanded_private_key.len(),
        64,
        "Invalid expanded private key length: expected 64 bytes, got {} bytes",
        jwt_expanded_private_key.len()
    );
    (jwt_public_key, jwt_expanded_private_key)
}

/// The configuration environment variables
#[derive(Clone, Deserialize)]
struct ConfigEnvVars {
    pub config_file: Option<PathBuf>,
    pub web_base_url: Option<Url>,
}

/// The configuration file format
#[derive(Clone, Deserialize)]
struct ConfigFile {
    /// The user-facing name of the service
    pub service_name: String,
    pub host: String,
    pub port: u16,
    pub openai_key: String,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_region: String,
    pub aws_image_id: String,
    pub github_git_name: String,
    pub github_git_email: String,
    pub github_app_id: String,
    pub github_app_client_id: String,
    pub github_app_client_secret: String,
    /// GitHub app webhook secret
    pub github_app_webhook_secret: String,
    /// Path to the GitHub app private key
    pub github_app_private_key: String,
    /// ID of the GitHub app installation on which the GitHub app is installed
    /// Currently, only a single installation is supported
    pub github_app_installation_id: i64,
    /// ID of the GitHub organization on which the GitHub app is installed
    pub github_app_organization_id: String,
    /// Name of the GitHub organization on which the GitHub app is installed
    pub github_app_organization_name: String,
    /// The handle of the GitHub bot including the leading '@', e.g. "@handle"
    pub github_bot_handle: String,
    /// The User-Agent string to use when making requests to the GitHub API
    pub github_api_user_agent: String,
    pub postgres_url: Url,
    pub web_base_url: Url,
    // Default agent configuration
    pub default_agent_container_registry_host: String,
    pub default_agent_container_registry_username: String,
    pub default_agent_container_registry_password: String,
    pub default_agent_container_image: String,
    // JWT secret keys
    pub jwt_private_key: PathBuf,
    pub jwt_public_key: PathBuf,
    /// Access control mode
    pub access_control: AccessControl,
    /// Email addresses that are on the allowlist
    pub allowed_emails: Option<Vec<String>>,
    /// Whether to dispatch jobs
    pub dispatch_mode: Option<DispatchMode>,
    pub s3_endpoint: Url,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_prefix: String,
    /// Directory of static files to serve
    pub static_dir: PathBuf,
}

#[derive(Clone, Deserialize)]
pub enum AccessControl {
    /// Only users with allowed email addresses can sign in
    Allowlist,
    /// All users can sign in, but are placed on a waitlist for manual approval
    Waitlist,
}

#[derive(Clone, Deserialize, Default)]
pub enum DispatchMode {
    /// Don't dispatch jobs. This is useful for local development.
    None,
    /// Dispatch jobs by launching an agent in an AWS EC2 virtual machine.
    #[default]
    AWS,
}
