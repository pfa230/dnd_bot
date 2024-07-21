use std::future::Future;

use anyhow::{anyhow, bail};
use tracing::instrument;

use crate::{
    callback::{
        make_manage_harm_keyboard, make_manage_players_keyboard, make_manage_stress_keyboard,
        make_manage_timers_keyboard, make_players_keyboard, make_timers_keyboard,
    },
    context::BotContext,
    tracker::{PlayersKeyboard, PlayersMsg, TimersMsg, Tracker},
    utils::{debug_err, Bot, MarkdownBot},
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
    pub context: BotContext,
    pub chat_id: ChatId,
    pub from: User,
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
                .ok_or(anyhow!("Cannot find \\'from\\' user"))?
                .to_owned(),
        })
    }

    #[instrument(skip(self))]
    pub async fn handle_wipe(&self, confirm: &str) -> anyhow::Result<()> {
        match confirm {
            "" => {
                self.send_response("Are you sure? If so, do `/wipe yes`".to_owned())
                    .await
            }
            "yes" => {
                self.context.put(&Tracker::new()).await?;
                self.send_response("*Wipe successful*".to_owned()).await?;
                Ok(())
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

    #[instrument(skip(self))]
    pub async fn handle_roll(&self, num: usize) -> anyhow::Result<()> {
        if num > 5 {
            bail!("Too many dice: {}", num);
        }
        for _ in 0..num {
            self.bot.send_dice(self.chat_id).await?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn handle_create_player(&self, name: &str) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        let name = name.trim();
        if name.is_empty() {
            self.markdown_bot
                .send_message(self.chat_id, "Player name is required")
                .await?;
            return Ok(());
        }
        tracker.create_player(name)?;
        self.ignore_errors(|| self.update_players(&tracker, true))
            .await;
        self.send_response(format!("Player *{}* added", escape(name)))
            .await?;
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_create_timer(&self, name: &str, start_val: u16) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        let name = name.trim();
        if name.is_empty() {
            self.markdown_bot
                .send_message(self.chat_id, "Timer name is required")
                .await?;
            return Ok(());
        }
        tracker.create_timer(name, start_val.into())?;
        self.ignore_errors(|| self.update_timers(&tracker, true))
            .await;
        self.send_response(format!("Timer *{}* added", escape(name)))
            .await?;
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_list_players(&self) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        if let Some(last_msg) = &tracker.players_msg {
            self.ignore_errors(|| async {
                self.bot
                    .delete_message(self.chat_id, last_msg.msg_id)
                    .await?;
                self.bot
                    .delete_message(self.chat_id, last_msg.kb_id)
                    .await?;
                Ok(())
            })
            .await;
        }
        let msg = self
            .markdown_bot
            .send_message(self.chat_id, self.format_players_msg(&tracker))
            .await?;
        let kb = self
            .markdown_bot
            .send_message(self.chat_id, "*Manage:*")
            .reply_markup(make_players_keyboard())
            .await?;

        tracker.players_msg = Some(PlayersMsg {
            msg_id: msg.id,
            kb_id: kb.id,
            active_keyboard: PlayersKeyboard::None,
        });
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_list_timers(&self) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        if let Some(timers_msg) = &tracker.timers_msg {
            self.ignore_errors(|| async {
                self.bot
                    .delete_message(self.chat_id, timers_msg.msg_id)
                    .await?;
                self.bot
                    .delete_message(self.chat_id, timers_msg.kb_id)
                    .await?;
                Ok(())
            })
            .await;
        }
        let msg = self
            .markdown_bot
            .send_message(self.chat_id, self.format_timers_msg(&tracker))
            .await?;
        let kb = self
            .markdown_bot
            .send_message(self.chat_id, "*Manage:*")
            .reply_markup(make_manage_timers_keyboard(&tracker.timers))
            .await?;

        tracker.timers_msg = Some(TimersMsg {
            msg_id: msg.id,
            kb_id: kb.id,
            keyboard_active: true,
        });
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_change_harm(&self, id: usize, val: i32) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;

        let player = tracker.change_harm(id, val)?;
        self.send_response(format!(
            "Player *{}* has *{}* harm",
            escape(&player.name),
            escape(&player.harm.to_string())
        ))
        .await?;
        self.ignore_errors(|| self.update_players(&tracker, false))
            .await;
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_change_stress(&self, id: usize, val: i32) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;

        let player = tracker.change_stress(id, val)?;
        self.send_response(format!(
            "Player *{}* has *{}* stress",
            escape(&player.name),
            escape(&player.stress.to_string())
        ))
        .await?;
        self.ignore_errors(|| self.update_players(&tracker, false))
            .await;
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_change_timer(&self, id: usize, val: i32) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;

        let timer = tracker.change_timer(id, val)?;
        if timer.value <= 0 {
            tracker.delete_timer(id)?;
            self.send_response(format!("Timer *{}* has fired\\!", escape(&timer.name)))
                .await?;
            self.ignore_errors(|| self.update_timers(&tracker, true))
                .await;
        } else {
            self.send_response(format!(
                "Timer *{}* has *{}* ticks left",
                escape(&timer.name),
                escape(&timer.value.to_string())
            ))
            .await?;
            self.ignore_errors(|| self.update_timers(&tracker, false))
                .await;
        }
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_delete_player(&self, id: usize) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        let timer = tracker.delete_player(id)?;
        self.send_response(format!(
            "Player *{}* with *{}* harm and *{}* stress has been removed",
            escape(&timer.name),
            escape(&timer.harm.to_string()),
            escape(&timer.stress.to_string())
        ))
        .await?;
        self.ignore_errors(|| self.update_players(&tracker, true))
            .await;
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_delete_timer(&self, id: usize) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        let timer = tracker.delete_timer(id)?;
        self.send_response(format!(
            "Timer *{}* with *{}* ticks has beed removed",
            escape(&timer.name),
            escape(&timer.value.to_string())
        ))
        .await?;
        self.ignore_errors(|| self.update_timers(&tracker, true))
            .await;
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_show_timers_kb(&self) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        if let Some(timers_msg) = tracker.timers_msg.as_mut() {
            timers_msg.keyboard_active = true;
            self.update_timers_kb(&tracker).await?;
        }
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_hide_timers_kb(&self) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        if let Some(timers_msg) = tracker.timers_msg.as_mut() {
            timers_msg.keyboard_active = false;
            self.update_timers_kb(&tracker).await?;
        }
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_show_players_kb(&self) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        if let Some(players_msg) = tracker.players_msg.as_mut() {
            players_msg.active_keyboard = PlayersKeyboard::ManagePlayers;
            self.update_players_kb(&tracker, true).await?;
        }
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_show_harm_kb(&self) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        if let Some(players_msg) = tracker.players_msg.as_mut() {
            players_msg.active_keyboard = PlayersKeyboard::Harm;
            self.update_players_kb(&tracker, true).await?;
        }
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_show_stress_kb(&self) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        if let Some(players_msg) = tracker.players_msg.as_mut() {
            players_msg.active_keyboard = PlayersKeyboard::Stress;
            self.update_players_kb(&tracker, true).await?;
        }
        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    pub async fn handle_hide_players_kb(&self) -> anyhow::Result<()> {
        let mut tracker = self.context.get().await?;
        if let Some(players_msg) = tracker.players_msg.as_mut() {
            players_msg.active_keyboard = PlayersKeyboard::None;
            self.update_players_kb(&tracker, true).await?;
        }

        self.context.put(&tracker).await
    }

    #[instrument(skip(self))]
    async fn send_response(&self, text: String) -> anyhow::Result<()> {
        self.markdown_bot
            .send_message(
                self.chat_id,
                format!("{} by {}", text, markdown::user_mention_or_link(&self.from)),
            )
            .await?;
        Ok(())
    }

    pub fn format_user(&self) -> String {
        format!(
            "{}({})",
            &self.from.username.as_deref().unwrap_or("<>"),
            self.from.id
        )
    }

    #[instrument(skip(self, tracker))]
    async fn update_players(&self, tracker: &Tracker, update_kb: bool) -> anyhow::Result<()> {
        if let Some(last_msg) = tracker.players_msg.as_ref() {
            self.markdown_bot
                .edit_message_text(
                    self.chat_id,
                    last_msg.msg_id,
                    self.format_players_msg(tracker),
                )
                .await?;
            if update_kb {
                self.update_players_kb(tracker, false).await?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self, tracker))]
    async fn update_players_kb(
        &self,
        tracker: &Tracker,
        update_message: bool,
    ) -> anyhow::Result<()> {
        if let Some(last_msg) = tracker.players_msg.as_ref() {
            let (new_kb, manage_name) = match last_msg.active_keyboard {
                PlayersKeyboard::Harm => (make_manage_harm_keyboard(&tracker.players), " harm"),
                PlayersKeyboard::Stress => {
                    (make_manage_stress_keyboard(&tracker.players), " stress")
                }
                PlayersKeyboard::ManagePlayers => {
                    (make_manage_players_keyboard(&tracker.players), " players")
                }
                PlayersKeyboard::None => (make_players_keyboard(), ""),
            };
            if update_message {
                self.markdown_bot
                    .edit_message_text(
                        self.chat_id,
                        last_msg.kb_id,
                        format!("*Manage{manage_name}:*"),
                    )
                    .reply_markup(new_kb)
                    .await?;
            } else {
                self.markdown_bot
                    .edit_message_reply_markup(self.chat_id, last_msg.kb_id)
                    .reply_markup(new_kb)
                    .await?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self, tracker))]
    async fn update_timers(&self, tracker: &Tracker, update_kb: bool) -> anyhow::Result<()> {
        if let Some(last_msg) = tracker.timers_msg.as_ref() {
            self.markdown_bot
                .edit_message_text(
                    self.chat_id,
                    last_msg.msg_id,
                    self.format_timers_msg(tracker),
                )
                .await?;
            if update_kb {
                self.update_timers_kb(tracker).await?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self, tracker))]
    async fn update_timers_kb(&self, tracker: &Tracker) -> anyhow::Result<()> {
        if let Some(last_msg) = tracker.timers_msg.as_ref() {
            let kb = if last_msg.keyboard_active {
                make_manage_timers_keyboard(&tracker.timers)
            } else {
                make_timers_keyboard()
            };
            self.markdown_bot
                .edit_message_reply_markup(self.chat_id, last_msg.kb_id)
                .reply_markup(kb)
                .await?;
        }
        Ok(())
    }

    fn format_players_msg(&self, tracker: &Tracker) -> String {
        let mut out = String::new();
        out.push_str("*Players:*\n\n");
        for player in tracker.players.iter() {
            out.push_str(
                format!(
                    "*{}*: *{}* harm, *{}* stress\n",
                    escape(&player.name),
                    escape(&player.harm.to_string()),
                    escape(&player.stress.to_string())
                )
                .as_str(),
            );
        }
        out
    }

    fn format_timers_msg(&self, tracker: &Tracker) -> String {
        let mut out = String::new();
        out.push_str("*Timers:*\n\n");
        for timer in tracker.timers.iter() {
            out.push_str(
                format!(
                    "*{}*: *{}* ticks left\n",
                    escape(&timer.name),
                    escape(&timer.value.to_string()),
                )
                .as_str(),
            );
        }
        out
    }

    async fn ignore_errors<Fut, F>(&self, f: F)
    where
        F: Fn() -> Fut,
        Fut: Future<Output = anyhow::Result<()>>,
    {
        if let Err(err) = f().await {
            debug_err(&err).await;
        }
    }
}
