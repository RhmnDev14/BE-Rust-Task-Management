use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Builder;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use std::env;
use std::time::Duration;

#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub async fn new() -> Self {
        let endpoint_url = env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set");
        let access_key = env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set");
        let secret_key = env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set");
        let bucket = env::var("MINIO_BUCKET").expect("MINIO_BUCKET must be set");

        let credentials = Credentials::new(access_key, secret_key, None, None, "minio");
        
        let config = Builder::new()
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .region(Region::new("us-east-1")) // MinIO doesn't care much about region, but SDK needs it
            .endpoint_url(endpoint_url)
            .credentials_provider(credentials)
            .force_path_style(true) // Required for MinIO
            .build();

        let client = Client::from_conf(config);

        Self { client, bucket }
    }

    pub async fn generate_presigned_url(
        &self,
        file_name: &str,
        expires_in: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_in))?;

        let presigned_request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(file_name)
            .presigned(presigning_config)
            .await?;

        Ok(presigned_request.uri().to_string())
    }
}
