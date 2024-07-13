use ::tracing::{info, instrument};
use anyhow::Context;
use commands::{handle_command, is_command, Command};
use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestPayloadExt, Response};
use rand::seq::SliceRandom;
use teloxide::{
    prelude::*,
    types::UpdateKind,
    utils::command::BotCommands,
};
use tracing::warn;
use utils::{
    authorize, create_bot, error_response, init_context, is_maxim, success_response, Bot,
    BotContext,
};

mod commands;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .with_current_span(false)
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();

    info!("Starting bot...");

    // Cold start bot
    let bot = create_bot();
    // Set commands
    bot.set_my_commands(Command::bot_commands())
        .await
        .context("Error setting commands")?;

    let context = init_context(&bot).await?;

    run(service_fn(|req| {
        function_handler(bot.clone(), req, context.clone())
    }))
    .await
}

async fn function_handler(
    bot: Bot,
    event: Request,
    context: BotContext,
) -> Result<Response<Body>, Error> {
    if let Err(e) = authorize(&event) {
        return error_response(401, format!("Unauthorized: {e}"));
    }

    let update: Update = match event.payload() {
        Ok(Some(payload)) => payload,
        Ok(None) => {
            return error_response(400, "Empty payload");
        }
        Err(e) => {
            return error_response(400, format!("Invalid payload: {e}"));
        }
    };

    let msg = match update.kind {
        UpdateKind::Message(message) => message,
        _ => {
            return error_response(400, format!("Unsupported payload: {:?}", update.kind));
        }
    };

    match handle(&bot, msg, &context).await {
        Ok(()) => success_response(),
        Err(e) => error_response(400, format!("Error: {e}")),
    }
}

#[instrument(skip(bot, context))]
async fn handle(bot: &Bot, msg: Message, context: &BotContext) -> anyhow::Result<()> {
    if let Some(cmd) = is_command(&msg, context) {
        handle_command(bot, &msg, cmd).await
    } else if is_talking_to_bot(&msg, context) {
        match is_maxim(&msg) {
            true => handle_maxim(bot, &msg, context).await,
            false => handle_dialog(bot, &msg).await,
        }
    } else {
        info!("Message is not supported: {:?}", msg);
        Ok(())
    }
}

#[instrument(skip(bot, context))]
async fn handle_maxim(bot: &Bot, msg: &Message, context: &BotContext) -> anyhow::Result<()> {
    warn!("Maxim is here!");

    bot.send_message(msg.chat.id, "Привет, Максим!").await?;

    let sticker_file = context
        .petrosyan
        .choose(&mut rand::thread_rng());
    if let Some(f) = sticker_file {
        bot.send_sticker(msg.chat.id, f.clone()).await?;
    }
    
    Ok(())
}

fn is_talking_to_bot(msg: &Message, context: &BotContext) -> bool {
    let at_name = format!("@{}", context.bot_name);
    match msg.chat.kind {
        teloxide::types::ChatKind::Private(_) => true,
        teloxide::types::ChatKind::Public(_) => msg.text().map_or(false, |text| {
            text.split_ascii_whitespace().any(|token| at_name.eq(token))
        }),
    }
}

#[instrument(skip(bot))]
async fn handle_dialog(bot: &Bot, msg: &Message) -> anyhow::Result<()> {
    info!("Handling dialog");

    bot.send_message(msg.chat.id, "Are you talking to me?")
        .await?;
    Ok(())
}
