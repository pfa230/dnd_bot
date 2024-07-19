use std::sync::Arc;

use teloxide::types::ChatId;
use tokio::sync::Mutex;

use crate::tracker::{Item, Tracker};

#[derive(Clone)]
pub struct BotContext {
    timers: Arc<Mutex<Tracker>>,
    harm: Arc<Mutex<Tracker>>,
    stress: Arc<Mutex<Tracker>>,
}

impl BotContext {
    pub async fn new(chat_id: ChatId) -> Self {
        BotContext {
            timers: Arc::new(Mutex::new(Tracker::new("timers", chat_id).await)),
            harm: Arc::new(Mutex::new(Tracker::new("harm", chat_id).await)),
            stress: Arc::new(Mutex::new(Tracker::new("stress", chat_id).await)),
        }
    }

    pub async fn reset(&self) -> anyhow::Result<()> {
        self.timers.lock().await.reset().await?;
        self.harm.lock().await.reset().await?;
        self.stress.lock().await.reset().await?;
        Ok(())
    }

    // Timers
    pub async fn create_timer(&self, name: &str, start_value: u16) -> anyhow::Result<Item> {
        self.timers.lock().await.add(name, start_value.into()).await
    }

    pub async fn list_timers(&self) -> anyhow::Result<Vec<Item>> {
        self.timers.lock().await.list().await
    }

    pub async fn get_timer(&self, id: usize) -> anyhow::Result<Item> {
        self.timers.lock().await.get(id).await
    }

    pub async fn tick_timer(&self, id: usize) -> anyhow::Result<Option<Item>> {
        self.timers.lock().await.tick(id).await
    }

    pub async fn delete_timer(&self, id: usize) -> anyhow::Result<Item> {
        self.timers.lock().await.delete(id).await
    }

    // Harm
    pub async fn create_harm(&self, name: &str) -> anyhow::Result<Item> {
        self.harm.lock().await.add(name, 0).await
    }

    pub async fn list_harm(&self) -> anyhow::Result<Vec<Item>> {
        self.harm.lock().await.list().await
    }

    pub async fn change_harm(&self, id: usize, change: i32) -> anyhow::Result<Item> {
        self.harm.lock().await.change(id, change).await
    }

    pub async fn delete_harm(&self, id: usize) -> anyhow::Result<Item> {
        self.harm.lock().await.delete(id).await
    }

    // Stress
    pub async fn create_stress(&self, name: &str) -> anyhow::Result<Item> {
        self.stress.lock().await.add(name, 0).await
    }

    pub async fn list_stress(&self) -> anyhow::Result<Vec<Item>> {
        self.stress.lock().await.list().await
    }

    pub async fn change_stress(&self, id: usize, change: i32) -> anyhow::Result<Item> {
        self.stress.lock().await.change(id, change).await
    }

    pub async fn delete_stress(&self, id: usize) -> anyhow::Result<Item> {
        self.stress.lock().await.delete(id).await
    }
}
