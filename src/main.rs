use ::tracing::error;
use anyhow::anyhow;
use commands::{handle_command, is_command, Command};
use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestPayloadExt, Response};
use rand::seq::SliceRandom;
use teloxide::{
    prelude::*,
    types::{InputFile, UpdateKind},
    utils::command::BotCommands,
};
use tracing::{debug, warn};
use utils::{create_bot, Bot, Context};

mod commands;
mod utils;

const MAXIM_IDS: [u64; 1] = [112553360];

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();

    // Cold start bot
    let bot = create_bot();

    // Set commands
    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("Error setting commands");
    let stickers = bot
        .get_sticker_set("Smekhopanorama")
        .await
        .expect("Error getting stickers");
    let me = bot.get_me().await.expect("Error getting 'me'");
    let context = Context { stickers, me };

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
    match handle(&bot, event, &context).await {
        Ok(()) => Ok(lambda_http::Response::builder()
            .status(200)
            .body("".into())
            .unwrap()),
        Err(e) => {
            error!("{}", e);
            Ok(lambda_http::Response::builder()
                .status(400)
                .body(format!("Error: {}", e).into())
                .unwrap())
        }
    }
}

#[tracing::instrument(skip_all)]
async fn handle(bot: &Bot, event: Request, context: &Context) -> anyhow::Result<()> {
    let update: Update = event.payload()?.ok_or(anyhow!("Empty payload"))?;

    let msg = match update.kind {
        UpdateKind::Message(message) => message,
        _ => {
            debug!("Unsupported update: {:?}", update);
            return Ok(());
        }
    };

    if let Some(_) = is_maxim(&msg) {
        handle_maxim(bot, &msg, context).await
    } else if let Some(cmd) = is_command(&msg, context) {
        handle_command(bot, &msg, cmd).await
    } else {
        handle_dialog(bot, &msg).await
    }
}

fn is_maxim(msg: &Message) -> Option<()> {
    msg.from()
        .map(|user| MAXIM_IDS.iter().any(|maxim| *maxim == user.id.0))
        .unwrap_or_default()
        .then(|| ())
}

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

async fn handle_dialog(bot: &Bot, msg: &Message) -> anyhow::Result<()> {
    debug!("Received msg: {:?}", msg);

    bot.send_message(msg.chat.id, "Not today.").await?;
    Ok(())
}
