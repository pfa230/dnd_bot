use anyhow::anyhow;
use teloxide::{
    prelude::*,
    requests::Requester,
    types::{Message, UpdateKind},
    utils::command::BotCommands,
};
use tracing::{info, instrument, warn};

use crate::callback::Callback;
use crate::callback::CallbackAction;
use crate::handler::BotHandler;
use crate::utils::Bot;

#[derive(BotCommands, PartialEq, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:",
    parse_with = "split"
)]
pub enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(description = "clears everything")]
    Reset(String),
    #[command(description = "rolls 1 die")]
    R1,
    #[command(description = "rolls 2 dice")]
    R2,
    #[command(description = "rolls 3 dice")]
    R3,
    #[command(description = "manage timers")]
    T,
    #[command(description = "`<name> <start_value>` - add timer")]
    Ta(String, u16),
    #[command(description = "manage harm")]
    A,
    #[command(description = "`<name>` - add harm recipient")]
    Aa(String),
    #[command(description = "manage stress")]
    S,
    #[command(description = "`<name>` - add stress recipient")]
    Sa(String),
}

#[instrument(skip(bot))]
pub async fn dispatch_update(bot: Bot, update: Update) -> anyhow::Result<()> {
    info!("Handle update called with {:?}", update);
    let ret = match &update.kind {
        // UpdateKind::InlineQuery(inline) => handle_inline(bot, inline, context).await,
        UpdateKind::Message(msg) => dispatch_command(&bot, &msg).await,
        UpdateKind::CallbackQuery(cb) => dispatch_callback(&bot, &cb).await,
        _ => {
            warn!("Unsupported update kind: {:?}", update.kind);
            Ok(())
        }
    };
    if let Err(err) = &ret {
        warn!("Error handling update: {:?}", err);
        if let Some(chat) = update.chat() {
            let _ = bot
                .send_message(
                    chat.id,
                    format!("Error handling update {}: {}", update.id.0, err),
                )
                .await;
        }
    }
    Ok(())
}

#[instrument(skip(bot))]
pub async fn dispatch_callback(bot: &Bot, cb: &CallbackQuery) -> anyhow::Result<()> {
    let data = cb.data.as_deref().ok_or(anyhow!("Missing callback data"))?;
    info!("Handling callback '{}'", data);

    let handler = BotHandler::new(bot.clone()).await;
    let callback = Callback::deserialize(data)?;

    match callback.action {
        CallbackAction::TickTimer => {
            handler
                .handle_tick_timer(callback.chat_id, callback.id)
                .await
        }
        CallbackAction::DeleteTimer => {
            handler
                .handle_delete_timer(callback.chat_id, callback.id)
                .await
        }
        CallbackAction::AddHarm => {
            handler
                .handle_change_harm(callback.chat_id, callback.id, 1)
                .await
        }
        CallbackAction::SubHarm => {
            handler
                .handle_change_harm(callback.chat_id, callback.id, -1)
                .await
        }
        CallbackAction::DeleteHarm => {
            handler
                .handle_delete_harm(callback.chat_id, callback.id)
                .await
        }
        CallbackAction::AddStress => {
            handler
                .handle_change_stress(callback.chat_id, callback.id, 1)
                .await
        }
        CallbackAction::SubStress => {
            handler
                .handle_change_stress(callback.chat_id, callback.id, -1)
                .await
        }
        CallbackAction::DeleteStress => {
            handler
                .handle_delete_stress(callback.chat_id, callback.id)
                .await
        }
    }
}

#[instrument(skip(bot))]
pub async fn dispatch_command(bot: &Bot, msg: &Message) -> anyhow::Result<()> {
    let text = msg.text().ok_or(anyhow!("Error parsing command"))?;
    info!("Received command '{}'", text);

    let handler = BotHandler::new(bot.clone()).await;
    match Command::parse(text, bot.get_me().await?.username())? {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
            Ok(())
        }
        Command::Reset(confirm) => handler.handle_reset(msg.chat.id, &confirm).await,
        Command::R1 => handler.handle_roll(msg.chat.id, 1).await,
        Command::R2 => handler.handle_roll(msg.chat.id, 2).await,
        Command::R3 => handler.handle_roll(msg.chat.id, 3).await,
        Command::T => handler.handle_list_timers(msg.chat.id).await,
        Command::Ta(name, start_val) => {
            handler
                .handle_create_timer(msg.chat.id, &name, start_val)
                .await
        }
        Command::A => handler.handle_list_harm(msg.chat.id).await,
        Command::Aa(name) => handler.handle_create_harm(msg.chat.id, &name).await,
        Command::S => handler.handle_list_stress(msg.chat.id).await,
        Command::Sa(name) => handler.handle_create_stress(msg.chat.id, &name).await,
    }
}
