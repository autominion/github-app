#![cfg(test)]

use std::env;

use uuid::Uuid;

use super::{GetObjectError, S3Config, S3};

fn s3_config() -> S3Config {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(dotenvy::Error::Io(err)) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                panic!("Failed to load .env file: {}", err);
            } else {
                eprintln!("No .env file found, using environment variables");
            }
        }
        Err(err) => panic!("Failed to load .env file: {}", err),
    }

    S3Config {
        bucket: env::var("TEST_S3_BUCKET").expect("TEST_S3_BUCKET must be set"),
        prefix: env::var("TEST_S3_PREFIX").expect("TEST_S3_PREFIX must be set"),
        region: env::var("TEST_S3_REGION").expect("TEST_S3_REGION must be set"),
        endpoint: env::var("TEST_S3_ENDPOINT")
            .expect("TEST_S3_ENDPOINT must be set")
            .parse()
            .expect("Failed to parse TEST_S3_ENDPOINT"),
        access_key: env::var("TEST_S3_ACCESS_KEY").expect("TEST_S3_ACCESS_KEY must be set"),
        secret_key: env::var("TEST_S3_SECRET_KEY").expect("TEST_S3_SECRET_KEY must be set"),
    }
}

#[tokio::test]
async fn test_put_then_get_log() {
    let config = s3_config();

    let s3 = S3::new(config).expect("Failed to create S3 client");

    let task_id = "01954d93-5ca7-791d-859d-70775efb9b69".parse::<Uuid>().unwrap();

    let log_content = "Test log content for integration test".to_owned();

    s3.upload_log_for_task(&task_id, log_content.clone()).await.expect("Failed to upload log");

    let retrieved = s3.log_for_task(&task_id).await.expect("Failed to retrieve log");

    assert_eq!(retrieved, log_content.into_bytes());

    s3.delete(&format!("tasks/{}/task.log", task_id)).await.expect("Failed to delete log");
}

#[tokio::test]
async fn test_get_non_existent_log() {
    let config = s3_config();

    let s3 = S3::new(config).expect("Failed to create S3 client");

    let task_id = "01954d93-8fd1-7d34-8a0b-07e21424db10".parse::<Uuid>().unwrap();

    let err = s3.log_for_task(&task_id).await.unwrap_err();

    assert!(matches!(err, GetObjectError::NotFound));
}
