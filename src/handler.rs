use anyhow::bail;

use crate::{
    callback::{make_harm_keyboard, make_stress_keyboard, make_timers_keyboard},
    context::BotContext,
    utils::Bot,
};
use teloxide::{payloads::SendMessageSetters, prelude::*};

pub struct BotHandler {
    bot: Bot,
    context: BotContext,
}

impl BotHandler {
    pub async fn new(bot: Bot) -> Self {
        Self {
            bot,
            context: BotContext::new().await,
        }
    }

    pub async fn handle_reset(&self, chat_id: ChatId, confirm: &str) -> anyhow::Result<()> {
        match confirm {
            "" => {
                self.bot.send_message(chat_id, "Are you sure? If so, do `/reset yes`").await?;
            }
            "yes" => {
                self.context.reset().await?;
                self.bot.send_message(chat_id, "*Reset successful*").await?;
            }
            str => {
                self.bot
                    .send_message(
                        chat_id,
                        format!(
                            "Only 'yes' is accepted as confirmation, but received {}",
                            str
                        ),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn handle_roll(&self, chat_id: ChatId, num: usize) -> anyhow::Result<()> {
        if num > 5 {
            bail!("Too many dice: {}", num);
        }
        for _ in 0..num {
            self.bot.send_dice(chat_id).await?;
        }
        Ok(())
    }

    pub async fn handle_list_timers(&self, chat_id: ChatId) -> anyhow::Result<()> {
        let timers = self.context.list_timers().await?;
        let mut out = String::new();
        out.push_str("*Active timers:*\n");
        for timer in timers.iter() {
            out.push_str(format!("*{}*: *{}* ticks left\n", timer.name, timer.value).as_str());
        }
        self.bot
            .send_message(chat_id, out)
            .reply_markup(make_timers_keyboard(chat_id, &timers))
            .await?;
        Ok(())
    }

    pub async fn handle_tick_timer(&self, chat_id: ChatId, id: usize) -> anyhow::Result<()> {
        let timer = self.context.get_timer(id).await?;
        match self.context.tick_timer(id).await? {
            Some(timer) => {
                self.bot
                    .send_message(
                        chat_id,
                        format!("Timer {} has *{}* ticks left", &timer.name, timer.value),
                    )
                    .await?;
            }
            None => {
                self.bot
                    .send_message(chat_id, format!("Timer *{}* has fired!", &timer.name))
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn handle_create_timer(
        &self,
        chat_id: ChatId,
        name: &str,
        start_val: u16,
    ) -> anyhow::Result<()> {
        self.context.create_timer(name, start_val).await?;
        self.bot
            .send_message(chat_id, format!("Timer *{}* added", name))
            .await?;
        Ok(())
    }

    pub async fn handle_delete_timer(&self, chat_id: ChatId, id: usize) -> anyhow::Result<()> {
        let timer = self.context.delete_timer(id).await?;
        self.bot
            .send_message(chat_id, format!("Timer *{}* removed.", &timer.name))
            .await?;
        Ok(())
    }

    pub async fn handle_list_harm(&self, chat_id: ChatId) -> anyhow::Result<()> {
        let all_harm = self.context.list_harm().await?;
        let mut out = String::new();
        out.push_str("*Harm:*\n");
        for harm in all_harm.iter() {
            out.push_str(format!("*{}* has *{}* harm\n", harm.name, harm.value).as_str());
        }
        self.bot
            .send_message(chat_id, out)
            .reply_markup(make_harm_keyboard(chat_id, &all_harm))
            .await?;
        Ok(())
    }

    pub async fn handle_create_harm(&self, chat_id: ChatId, name: &str) -> anyhow::Result<()> {
        self.context.create_harm(name).await?;
        self.bot
            .send_message(chat_id, format!("Harm recipient *{}* added", name))
            .await?;
        Ok(())
    }

    pub async fn handle_change_harm(
        &self,
        chat_id: ChatId,
        id: usize,
        change: i32,
    ) -> anyhow::Result<()> {
        let new_harm = self.context.change_harm(id, change).await?;
        self.bot
            .send_message(chat_id, format!("*{}* now has *{}* harm", &new_harm.name, new_harm.value))
            .await?;
        Ok(())
    }

    pub async fn handle_delete_harm(&self, chat_id: ChatId, id: usize) -> anyhow::Result<()> {
        let harm = self.context.delete_harm(id).await?;
        self.bot
            .send_message(chat_id, format!("Harm recipient *{}* removed.", &harm.name))
            .await?;
        Ok(())
    }

    pub async fn handle_list_stress(&self, chat_id: ChatId) -> anyhow::Result<()> {
        let all_stress = self.context.list_stress().await?;
        let mut out = String::new();
        out.push_str("Stress:\n");
        for stress in all_stress.iter() {
            out.push_str(format!("*{}* has *{}* stress\n", stress.name, stress.value).as_str());
        }
        self.bot
            .send_message(chat_id, out)
            .reply_markup(make_stress_keyboard(chat_id, &all_stress))
            .await?;
        Ok(())
    }

    pub async fn handle_create_stress(&self, chat_id: ChatId, name: &str) -> anyhow::Result<()> {
        self.context.create_stress(name).await?;
        self.bot
            .send_message(chat_id, format!("Stress recipient *{}* added", name))
            .await?;
        Ok(())
    }

    pub async fn handle_change_stress(
        &self,
        chat_id: ChatId,
        id: usize,
        change: i32,
    ) -> anyhow::Result<()> {
        let new_stress = self.context.change_stress(id, change).await?;
        self.bot
            .send_message(
                chat_id,
                format!("*{}* now has *{}* stress", &new_stress.name, new_stress.value),
            )
            .await?;
        Ok(())
    }

    pub async fn handle_delete_stress(&self, chat_id: ChatId, id: usize) -> anyhow::Result<()> {
        let stress = self.context.delete_stress(id).await?;
        self.bot
            .send_message(chat_id, format!("Stress recipient *{}* removed.", &stress.name))
            .await?;
        Ok(())
    }
}
