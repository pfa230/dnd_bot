use ::tracing::{info, instrument};
use commands::{handle_command, is_command, Command};
use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestPayloadExt, Response};
use rand::seq::SliceRandom;
use teloxide::{
    prelude::*,
    types::{InputFile, UpdateKind},
    utils::command::BotCommands,
};
use tracing::warn;
use utils::{authorize, create_bot, error_response, success_response, Bot, Context};

mod commands;
mod utils;

const MAXIM_IDS: [u64; 1] = [112553360];

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
        .expect("Error setting commands");

    // Populate context
    let stickers = bot
        .get_sticker_set("Smekhopanorama")
        .await
        .expect("Error getting stickers");
    let me = bot.get_me().await.expect("Error getting 'me'");
    let bot_name = me
        .username
        .as_deref()
        .expect("Bots must have a username")
        .to_owned();
    let context = Context { stickers, bot_name };

    run(service_fn(|req| {
        function_handler(bot.clone(), req, context.clone())
    }))
    .await
}

async fn function_handler(
    bot: Bot,
    event: Request,
    context: Context,
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
async fn handle(bot: &Bot, msg: Message, context: &Context) -> anyhow::Result<()> {
    if let Some(cmd) = is_command(&msg, context) {
        match is_maxim(&msg) {
            Some(()) => handle_maxim(bot, &msg, context).await,
            None => handle_command(bot, &msg, cmd).await,
        }
    } else if is_talking_to_bot(&msg, context) {
        match is_maxim(&msg) {
            Some(()) => handle_maxim(bot, &msg, context).await,
            None => handle_dialog(bot, &msg).await,
        }
    } else {
        info!("Message is not supported: {:?}", msg);
        Ok(())
    }
}

fn is_maxim(msg: &Message) -> Option<()> {
    msg.from()
        .map(|user| MAXIM_IDS.iter().any(|maxim| *maxim == user.id.0))
        .unwrap_or_default()
        .then(|| ())
}

#[instrument(skip(bot, context))]
async fn handle_maxim(bot: &Bot, msg: &Message, context: &Context) -> anyhow::Result<()> {
    warn!("Maxim is here!");

    let sticker_file = context
        .stickers
        .stickers
        .choose(&mut rand::thread_rng())
        .map(|sticker| InputFile::file_id(sticker.file.id.to_string()));
    match sticker_file {
        Some(f) => bot.send_sticker(msg.chat.id, f).await?,
        None => bot.send_message(msg.chat.id, "Not today").await?,
    };
    Ok(())
}

fn is_talking_to_bot(msg: &Message, context: &Context) -> bool {
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
