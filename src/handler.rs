use anyhow::{anyhow, bail};

use crate::{
    callback::{make_harm_keyboard, make_stress_keyboard, make_timers_keyboard},
    context::BotContext,
    utils::{Bot, MarkdownBot},
};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{ParseMode, User},
    utils::markdown::{self, escape},
};

pub struct BotHandler {
    pub bot: Bot,
    pub markdown_bot: MarkdownBot,

    context: BotContext,
    chat_id: ChatId,
    from: User,
}

impl BotHandler {
    pub async fn new(bot: Bot, update: &Update) -> anyhow::Result<Self> {
        let chat_id = update.chat().ok_or(anyhow!("Chat not found"))?.id;
        Ok(Self {
            bot: bot.clone(),
            markdown_bot: bot.parse_mode(ParseMode::MarkdownV2),
            context: BotContext::new(chat_id).await,
            chat_id,
            from: update
                .from()
                .ok_or(anyhow!("Cannot find 'from' user"))?
                .to_owned(),
        })
    }

    pub async fn handle_reset(&self, confirm: &str) -> anyhow::Result<()> {
        match confirm {
            "" => {
                self.send_response("Are you sure? If so, do `/reset yes`".to_owned())
                    .await
            }
            "yes" => {
                self.context.reset().await?;
                self.send_response("*Reset successful*".to_owned()).await
            }
            str => {
                self.send_response(format!(
                    "Only 'yes' is accepted as confirmation, but received {}",
                    str
                ))
                .await
            }
        }
    }

    pub async fn handle_roll(&self, num: usize) -> anyhow::Result<()> {
        if num > 5 {
            bail!("Too many dice: {}", num);
        }
        for _ in 0..num {
            self.bot.send_dice(self.chat_id).await?;
        }
        Ok(())
    }

    pub async fn handle_list_timers(&self) -> anyhow::Result<()> {
        let timers = self.context.list_timers().await?;
        let mut out = String::new();
        out.push_str("*Active timers:*\n\n");
        for timer in timers.iter() {
            out.push_str(format!("*{}*: *{}* ticks left\n", timer.name, timer.value).as_str());
        }
        self.bot
            .send_message(self.chat_id, out)
            .reply_markup(make_timers_keyboard(self.chat_id, &timers))
            .await?;
        Ok(())
    }

    pub async fn handle_tick_timer(&self, id: usize) -> anyhow::Result<()> {
        let timer = self.context.get_timer(id).await?;
        match self.context.tick_timer(id).await? {
            Some(timer) => {
                self.send_response(format!(
                    "Timer {} has *{}* ticks left",
                    &timer.name, timer.value
                ))
                .await
            }
            None => {
                self.send_response(format!("Timer *{}* has fired!", &timer.name))
                    .await
            }
        }
    }

    pub async fn handle_create_timer(&self, name: &str, start_val: u16) -> anyhow::Result<()> {
        self.context.create_timer(name, start_val).await?;
        self.send_response(format!("Timer *{}* added", name)).await
    }

    pub async fn handle_delete_timer(&self, id: usize) -> anyhow::Result<()> {
        let timer = self.context.delete_timer(id).await?;
        self.send_response(format!("Timer *{}* removed", &timer.name))
            .await
    }

    pub async fn handle_list_harm(&self) -> anyhow::Result<()> {
        let all_harm = self.context.list_harm().await?;
        let mut out = String::new();
        out.push_str("*Harm:*\n\n");
        for harm in all_harm.iter() {
            out.push_str(format!("*{}* has *{}* harm\n", harm.name, harm.value).as_str());
        }
        self.bot
            .send_message(self.chat_id, out)
            .reply_markup(make_harm_keyboard(self.chat_id, &all_harm))
            .await?;
        Ok(())
    }

    pub async fn handle_create_harm(&self, name: &str) -> anyhow::Result<()> {
        self.context.create_harm(name).await?;
        self.send_response(format!("Harm recipient *{}* added", name))
            .await
    }

    pub async fn handle_change_harm(&self, id: usize, change: i32) -> anyhow::Result<()> {
        let new_harm = self.context.change_harm(id, change).await?;
        self.send_response(format!(
            "*{}* now has *{}* harm",
            &new_harm.name, new_harm.value
        ))
        .await
    }

    pub async fn handle_delete_harm(&self, id: usize) -> anyhow::Result<()> {
        let harm = self.context.delete_harm(id).await?;
        self.send_response(format!("Harm recipient *{}* removed", &harm.name))
            .await
    }

    pub async fn handle_list_stress(&self) -> anyhow::Result<()> {
        let all_stress = self.context.list_stress().await?;
        let mut out = String::new();
        out.push_str("*Stress:*\n\n");
        for stress in all_stress.iter() {
            out.push_str(format!("*{}* has *{}* stress\n", stress.name, stress.value).as_str());
        }
        self.bot
            .send_message(self.chat_id, out)
            .reply_markup(make_stress_keyboard(self.chat_id, &all_stress))
            .await?;
        Ok(())
    }

    pub async fn handle_create_stress(&self, name: &str) -> anyhow::Result<()> {
        self.context.create_stress(name).await?;
        self.send_response(format!("Stress recipient *{}* added", name))
            .await
    }

    pub async fn handle_change_stress(&self, id: usize, change: i32) -> anyhow::Result<()> {
        let new_stress = self.context.change_stress(id, change).await?;
        self.send_response(format!(
            "*{}* now has *{}* stress",
            &new_stress.name, new_stress.value
        ))
        .await
    }

    pub async fn handle_delete_stress(&self, id: usize) -> anyhow::Result<()> {
        let stress = self.context.delete_stress(id).await?;
        self.send_response(format!("Stress recipient *{}* removed", &stress.name))
            .await
    }

    async fn send_response(&self, text: String) -> anyhow::Result<()> {
        self.markdown_bot
            .send_message(
                self.chat_id,
                escape(&format!("{} by {}", text, markdown::user_mention_or_link(&self.from))),
            )
            .await?;
        Ok(())
    }
}
