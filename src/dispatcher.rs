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
    #[command(description = "`\\<name\\> \\<start_value\\>` \\- add timer")]
    Ta(String, u16),
    #[command(description = "manage harm")]
    A,
    #[command(description = "`\\<name\\>` \\- add harm recipient")]
    Aa(String),
    #[command(description = "manage stress")]
    S,
    #[command(description = "`\\<name\\>` \\- add stress recipient")]
    Sa(String),
}

#[instrument(skip(bot))]
pub async fn dispatch_update(bot: Bot, update: Update) -> anyhow::Result<()> {
    info!("Handle update called with {:?}", update);
    let handler = BotHandler::new(bot, &update).await?;
    let ret = match &update.kind {
        // UpdateKind::InlineQuery(inline) => handle_inline(bot, inline, context).await,
        UpdateKind::Message(msg) => dispatch_command(&handler, &msg).await,
        UpdateKind::CallbackQuery(cb) => dispatch_callback(&handler, &cb).await,
        _ => {
            warn!("Unsupported update kind: {:?}", update.kind);
            Ok(())
        }
    };
    if let Err(err) = &ret {
        warn!("Error handling update: {:?}", err);
        if let Some(chat) = update.chat() {
            let _ = handler.bot
                .send_message(
                    chat.id,
                    format!("Error handling update {}: {}", update.id.0, err),
                )
                .await;
        }
    }
    Ok(())
}

#[instrument(skip(handler))]
pub async fn dispatch_callback(handler: &BotHandler, cb: &CallbackQuery) -> anyhow::Result<()> {
    let data = cb.data.as_deref().ok_or(anyhow!("Missing callback data"))?;
    info!("Handling callback '{}'", data);

    let callback = Callback::deserialize(data)?;

    match callback.action {
        CallbackAction::TickTimer => {
            handler
                .handle_tick_timer(callback.id)
                .await
        }
        CallbackAction::DeleteTimer => {
            handler
                .handle_delete_timer(callback.id)
                .await
        }
        CallbackAction::AddHarm => {
            handler
                .handle_change_harm(callback.id, 1)
                .await
        }
        CallbackAction::SubHarm => {
            handler
                .handle_change_harm(callback.id, -1)
                .await
        }
        CallbackAction::DeleteHarm => {
            handler
                .handle_delete_harm(callback.id)
                .await
        }
        CallbackAction::AddStress => {
            handler
                .handle_change_stress(callback.id, 1)
                .await
        }
        CallbackAction::SubStress => {
            handler
                .handle_change_stress(callback.id, -1)
                .await
        }
        CallbackAction::DeleteStress => {
            handler
                .handle_delete_stress(callback.id)
                .await
        }
    }
}

#[instrument(skip(handler))]
pub async fn dispatch_command(handler: &BotHandler, msg: &Message) -> anyhow::Result<()> {
    let text = msg.text().ok_or(anyhow!("Error parsing command"))?;
    info!("Received command '{}'", text);

    match Command::parse(text, handler.bot.get_me().await?.username())? {
        Command::Help => {
            handler.bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
            Ok(())
        }
        Command::Reset(confirm) => handler.handle_reset(&confirm).await,
        Command::R1 => handler.handle_roll(1).await,
        Command::R2 => handler.handle_roll(2).await,
        Command::R3 => handler.handle_roll(3).await,
        Command::T => handler.handle_list_timers().await,
        Command::Ta(name, start_val) => {
            handler
                .handle_create_timer(&name, start_val)
                .await
        }
        Command::A => handler.handle_list_harm().await,
        Command::Aa(name) => handler.handle_create_harm(&name).await,
        Command::S => handler.handle_list_stress().await,
        Command::Sa(name) => handler.handle_create_stress(&name).await,
    }
}
