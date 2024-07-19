use anyhow::{anyhow, bail, Context};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{operation::get_object::GetObjectError, primitives::ByteStream, Client};
use serde::{Deserialize, Serialize};
use std::env;
use teloxide::types::ChatId;
use tracing::warn;

const S3_BUCKET_ENV_VAR: &str = "S3_BUCKET";

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Item {
    // Should be first for sorting purposes
    pub name: String,
    pub id: usize,
    pub value: i32,
}

#[derive(Clone, Serialize, Deserialize)]
struct Items(Vec<Item>);

#[derive(Clone)]
pub struct Tracker {
    client: Client,
    s3_path: String,
}

impl Tracker {
    pub async fn new(name: &str, chat_id: ChatId) -> Self {
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        Self {
            client: Client::new(&config),
            s3_path: format!("{}/{}.json", chat_id, name.to_owned()),
        }
    }

    pub async fn add(&mut self, name: &str, start_value: i32) -> anyhow::Result<Item> {
        let mut items = self.get_from_s3().await?;

        let item = items.add(name, start_value)?;
        self.put_to_s3(&items).await?;
        Ok(item)
    }

    pub async fn list(&self) -> anyhow::Result<Vec<Item>> {
        Ok(self.get_from_s3().await?.0)
    }

    pub async fn get(&self, id: usize) -> anyhow::Result<Item> {
        self.get_from_s3().await?.get(id)
    }

    pub async fn change(&mut self, id: usize, amount: i32) -> anyhow::Result<Item> {
        let mut items = self.get_from_s3().await?;
        let ret = items.change(id, amount);
        self.put_to_s3(&items).await?;
        ret
    }

    pub async fn delete(&mut self, id: usize) -> anyhow::Result<Item> {
        let mut items = self.get_from_s3().await?;
        let ret = items.delete(id);
        self.put_to_s3(&items).await?;
        ret
    }

    pub async fn tick(&mut self, id: usize) -> anyhow::Result<Option<Item>> {
        let item = self.change(id, -1).await?;
        if item.value <= 0 {
            self.delete(id).await?;
            Ok(None)
        } else {
            Ok(Some(item))
        }
    }

    pub async fn reset(&self) -> anyhow::Result<()> {
        let bucket = env::var(S3_BUCKET_ENV_VAR).unwrap();
        self.client
            .delete_object()
            .bucket(bucket)
            .key(&self.s3_path)
            .send()
            .await?;
        Ok(())
    }

    async fn get_from_s3(&self) -> anyhow::Result<Items> {
        let bucket = env::var(S3_BUCKET_ENV_VAR).unwrap();

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
                    GetObjectError::NoSuchKey(_) => Ok(Items(Vec::new())),
                    err => Err(err),
                }
            }
        }
        .with_context(|| "Error fetching from S3")
    }

    async fn put_to_s3(&self, items: &Items) -> anyhow::Result<()> {
        let bucket = env::var(S3_BUCKET_ENV_VAR).unwrap();

        self.client
            .put_object()
            .bucket(bucket)
            .key(&self.s3_path)
            .body(ByteStream::from(
                serde_json::to_string_pretty(&items)?.as_bytes().to_owned(),
            ))
            .send()
            .await
            .with_context(|| "Error putting to S3")?;
        Ok(())
    }
}

impl Items {
    fn change(&mut self, id: usize, amount: i32) -> anyhow::Result<Item> {
        let item = self
            .0
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or(anyhow!("Item not found"))?;

        item.value = item
            .value
            .checked_add(amount)
            .ok_or(anyhow!("Overflow for item '{}'", item.name))?;
        Ok(item.clone())
    }

    fn get(&self, id: usize) -> anyhow::Result<Item> {
        self.0
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.clone())
            .ok_or(anyhow!("Item not found"))
    }

    fn delete(&mut self, id: usize) -> anyhow::Result<Item> {
        let pos = self
            .0
            .iter()
            .position(|item| item.id == id)
            .ok_or(anyhow!("Item not found"))?;
        Ok(self.0.remove(pos))
    }

    fn next_id(&self) -> usize {
        self.0
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .unwrap()
    }

    fn add(&mut self, name: &str, value: i32) -> anyhow::Result<Item> {
        if self.0.iter().find(|item| item.name.eq(name)).is_some() {
            bail!("Item {} already present", name);
        }
        let item = Item {
            id: self.next_id(),
            name: name.to_owned(),
            value,
        };
        self.0.push(item.clone());
        self.0.sort();
        Ok(item)
    }
}
