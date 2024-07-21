use std::env;

use anyhow::Context;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{operation::get_object::GetObjectError, primitives::ByteStream, Client};
use teloxide::types::ChatId;
use tracing::{info, instrument, warn};

use crate::tracker::Tracker;

const S3_BUCKET_ENV_VAR: &str = "S3_BUCKET";

pub struct BotContext {
    client: Client,
    s3_path: String,
}

impl BotContext {
    pub async fn new(chat_id: ChatId) -> Self {
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;

        let dir = if chat_id.0 < 0 {
            format!("_{}", -chat_id.0)
        } else {
            chat_id.to_string()
        };

        let s3_path = format!("{}/store.json", dir);
        Self {
            client: Client::new(&config),
            s3_path,
        }
    }

    pub async fn get(&self) -> anyhow::Result<Tracker> {
        self.get_from_s3().await
    }

    #[instrument(skip(self), fields(s3_path = %self.s3_path))]
    async fn get_from_s3(&self) -> anyhow::Result<Tracker> {
        let bucket = env::var(S3_BUCKET_ENV_VAR).unwrap();
        info!("Fetching from S3 bucket {}", bucket);

        let response = self
            .client
            .get_object()
            .bucket(bucket)
            .key(&self.s3_path)
            .send()
            .await;
        match response {
            Ok(response) => Ok(serde_json::from_slice(
                &response.body.collect().await?.to_vec(),
            )?),
            Err(sdk_err) => {
                warn!("Error fetching from S3: {:?}", sdk_err);
                match sdk_err.into_service_error() {
                    GetObjectError::NoSuchKey(_) => Ok(Tracker::new()),
                    err => Err(err),
                }
            }
        }
        .with_context(|| "Error fetching from S3")
    }

    #[instrument(skip_all, fields(s3_path = %self.s3_path))]
    pub async fn put(&self, tracker: &Tracker) -> anyhow::Result<()> {
        let bucket = env::var(S3_BUCKET_ENV_VAR).unwrap();
        info!("Writing to S3 bucket {}", bucket);

        self.client
            .put_object()
            .bucket(bucket)
            .key(&self.s3_path)
            .body(ByteStream::from(
                serde_json::to_string_pretty(tracker)?.as_bytes().to_owned(),
            ))
            .send()
            .await
            .with_context(|| "Error putting to S3")?;
        Ok(())
    }
}
