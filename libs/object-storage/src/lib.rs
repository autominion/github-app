use std::error::Error;

use url::Url;

use config::Config;
use thiserror::Error;
use uuid::Uuid;

mod tests;

// Our S3 client wrapper.
#[derive(Clone)]
pub struct S3 {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: String,
}

impl S3 {
    pub fn new<C: Into<S3Config>>(config: C) -> Result<Self, Box<aws_sdk_s3::Error>> {
        let config: S3Config = config.into();

        let s3_config = aws_sdk_s3::config::Builder::default()
            .region(aws_sdk_s3::config::Region::new(config.region.clone()))
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::v2024_03_28())
            .endpoint_url(config.endpoint.as_str())
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                config.access_key,
                config.secret_key,
                None,
                None,
                "Static",
            ))
            .build();

        let client = aws_sdk_s3::Client::from_conf(s3_config);

        Ok(Self { client, bucket: config.bucket.clone(), prefix: config.prefix.clone() })
    }

    pub async fn upload_log_for_task(
        &self,
        task_id: &Uuid,
        log_contents: String,
    ) -> Result<(), aws_sdk_s3::Error> {
        self.put(&format!("tasks/{}/task.log", task_id), log_contents.into_bytes()).await
    }

    pub async fn log_for_task(&self, task_id: &Uuid) -> Result<Vec<u8>, GetObjectError> {
        self.get(&format!("tasks/{}/task.log", task_id)).await
    }

    async fn put(&self, key: &str, data: Vec<u8>) -> Result<(), aws_sdk_s3::Error> {
        self.client
            .put_object()
            .bucket(self.bucket.clone())
            .key(format!("{}/{}", self.prefix, key))
            .body(data.into())
            .send()
            .await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, GetObjectError> {
        let resp = self
            .client
            .get_object()
            .bucket(self.bucket.clone())
            .key(format!("{}/{}", self.prefix, key))
            .send()
            .await?;

        let aggregated = resp.body.collect().await?;
        Ok(aggregated.into_bytes().to_vec())
    }

    #[cfg(test)]
    async fn delete(&self, key: &str) -> Result<(), aws_sdk_s3::Error> {
        self.client
            .delete_object()
            .bucket(self.bucket.clone())
            .key(format!("{}/{}", self.prefix, key))
            .send()
            .await?;
        Ok(())
    }
}

pub struct S3Config {
    pub region: String,
    pub endpoint: Url,
    pub bucket: String,
    pub prefix: String,
    pub access_key: String,
    pub secret_key: String,
}

impl From<&Config> for S3Config {
    fn from(config: &Config) -> Self {
        Self {
            region: config.s3_region.clone(),
            endpoint: config.s3_endpoint.clone(),
            bucket: config.s3_bucket.clone(),
            prefix: config.s3_prefix.clone(),
            access_key: config.s3_access_key.clone(),
            secret_key: config.s3_secret_key.clone(),
        }
    }
}

#[derive(Error, Debug)]
pub enum GetObjectError {
    #[error("object not found")]
    NotFound,
    #[error("unexpected error: {0}")]
    Unexpected(#[from] Box<dyn Error + Send + Sync>),
}

impl From<aws_sdk_s3::Error> for GetObjectError {
    fn from(err: aws_sdk_s3::Error) -> Self {
        match err {
            aws_sdk_s3::Error::NoSuchKey(_) | aws_sdk_s3::Error::NotFound(_) => {
                GetObjectError::NotFound
            }
            other => GetObjectError::Unexpected(Box::new(other)),
        }
    }
}

impl From<aws_smithy_types::byte_stream::error::Error> for GetObjectError {
    fn from(err: aws_smithy_types::byte_stream::error::Error) -> Self {
        GetObjectError::Unexpected(Box::new(err))
    }
}

impl
    From<
        aws_smithy_runtime_api::client::result::SdkError<
            aws_sdk_s3::operation::get_object::GetObjectError,
            aws_smithy_runtime_api::http::Response,
        >,
    > for GetObjectError
{
    fn from(
        err: aws_smithy_runtime_api::client::result::SdkError<
            aws_sdk_s3::operation::get_object::GetObjectError,
            aws_smithy_runtime_api::http::Response,
        >,
    ) -> Self {
        match err {
            aws_smithy_runtime_api::client::result::SdkError::ServiceError(err) => {
                match err.into_err() {
                    aws_sdk_s3::operation::get_object::GetObjectError::NoSuchKey(_) => {
                        GetObjectError::NotFound
                    }
                    other => GetObjectError::Unexpected(Box::new(other)),
                }
            }
            other => GetObjectError::Unexpected(Box::new(other)),
        }
    }
}
