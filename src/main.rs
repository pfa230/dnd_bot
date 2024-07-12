use commands::{handle_command, Command};
use rand::seq::SliceRandom;
use teloxide::{
    prelude::*,
    types::{InputFile, MessageKind, StickerSet},
};

mod commands;

const MAXIM_IDS: [u64; 1] = [112553360];

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();
    let stickers = bot.get_sticker_set("Smekhopanorama").await.unwrap();

    let handler = Update::filter_message()
        .branch(
            // Filter a maintainer by a user ID.
            dptree::filter(|msg: Message| {
                msg.from()
                    .map(|user| MAXIM_IDS.iter().any(|maxim| *maxim == user.id.0))
                    .unwrap_or_default()
            })
            .endpoint(handle_maxim),
        )
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_command),
        )
        .branch(
            dptree::filter(|msg: Message| matches!(msg.kind, MessageKind::Common(_)))
                .endpoint(dialog),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![stickers])
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn dialog(bot: Bot, msg: Message) -> Result<(), teloxide::RequestError> {
    log::debug!("Received msg: {:?}", msg);

    bot.send_message(msg.chat.id, "Not today.").await?;
    Ok(())
}

async fn handle_maxim(
    bot: Bot,
    msg: Message,
    stickers: StickerSet,
) -> Result<(), teloxide::RequestError> {
    log::warn!("Maxim is here!");

    let sticker_file = stickers
        .stickers
        .choose(&mut rand::thread_rng())
        .map(|sticker| InputFile::file_id(sticker.file.id.to_string()));
    match sticker_file {
        Some(f) => bot.send_sticker(msg.chat.id, f).await?,
        None => bot.send_message(msg.chat.id, "Not today").await?,
    };

    Ok(())
}
