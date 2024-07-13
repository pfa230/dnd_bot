use std::env;

use anyhow::{anyhow, bail, Context};
use lambda_http::{Body, Error, Request, Response};
use teloxide::{
    adaptors::{throttle::Limits, CacheMe, Throttle},
    requests::{Requester, RequesterExt},
    types::{InputFile, Message},
};
use tracing::{info, warn};

static SECRET_TOKEN_HEADER: &str = "x-telegram-bot-api-secret-token";
static SECRET_TOKEN_ENV_VAR: &str = "AUTH_TOKEN";
static MAXIM_ID_ENV_VAR: &str = "MAXIM_ID";

pub type Bot = CacheMe<Throttle<teloxide::Bot>>;

pub fn create_bot() -> Bot {
    teloxide::Bot::from_env()
        .throttle(Limits::default())
        .cache_me()
}

pub fn authorize(event: &Request) -> anyhow::Result<()> {
    let expected_token = env::var(SECRET_TOKEN_ENV_VAR)?;
    let token_header = event
        .headers()
        .get(SECRET_TOKEN_HEADER)
        .ok_or(anyhow!("No {SECRET_TOKEN_HEADER} found"))?;

    if !expected_token.eq(token_header) {
        bail!("Invalid token passed: {:?}", token_header);
    }
    Ok(())
}

pub fn error_response<T: Into<String>>(code: u16, message: T) -> Result<Response<Body>, Error> {
    warn!(
        "Returning error code: {}, message: {} ",
        code,
        message.into()
    );
    Ok(lambda_http::Response::builder()
        .status(code)
        .body(Body::Empty)
        .unwrap())
}

pub fn success_response() -> Result<Response<Body>, Error> {
    info!("Returning success");
    Ok(lambda_http::Response::builder()
        .status(200)
        .body(Body::Empty)
        .unwrap())
}

#[derive(Clone)]
pub struct BotContext {
    pub dice_sticker: InputFile,
    pub petrosyan: Vec<InputFile>,
    pub bot_name: String,
}

pub async fn init_context(bot: &Bot) -> anyhow::Result<BotContext> {
    let dice_sticker_set = bot
        .get_sticker_set("max_dice")
        .await
        .context("Error getting Max's dice")?;
    let dice_sticker = dice_sticker_set
        .stickers
        .get(0)
        .ok_or(anyhow!("Dice sticker set is empty"))?;

    let petrosyan = bot
        .get_sticker_set("Smekhopanorama")
        .await
        .context("Error getting petrosyan stickers")?;

    let me = bot.get_me().await.context("Error getting 'me'")?;
    let bot_name = me
        .username
        .as_deref()
        .ok_or(anyhow!("Bots must have a username"))?
        .to_owned();

    Ok(BotContext {
        dice_sticker: InputFile::file_id(dice_sticker.file.id.to_string()),
        petrosyan: petrosyan
            .stickers
            .iter()
            .map(|s| InputFile::file_id(s.file.id.clone()))
            .collect(),
        bot_name,
    })
}

pub fn is_maxim(msg: &Message) -> bool {
    let maxim_id = env::var(MAXIM_ID_ENV_VAR);
    match maxim_id {
        Ok(id) => msg.from().map_or(false, |u| id.eq(&u.id.to_string())),
        Err(e) => {
            warn!("Error getting Maxim ID: {:?}", e);
            false
        }
    }
}
