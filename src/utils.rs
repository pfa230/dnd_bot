use std::env;

use anyhow::{anyhow, bail};
use lambda_http::{Body, Error, Request, Response};
use teloxide::{
    adaptors::{CacheMe, DefaultParseMode},
    prelude::*,
    requests::RequesterExt,
    utils::command::BotCommands,
};
use tracing::{info, warn};

use crate::dispatcher::Command;

static SECRET_TOKEN_HEADER: &str = "x-telegram-bot-api-secret-token";
static SECRET_TOKEN_ENV_VAR: &str = "AUTH_TOKEN";
static DEBUG_CHAT_ID_ENV_VAR: &str = "DEBUG_CHAT_ID";

pub type Bot = CacheMe<teloxide::Bot>;
pub type MarkdownBot = DefaultParseMode<CacheMe<teloxide::Bot>>;

pub async fn init_bot() -> Bot {
    let bot = teloxide::Bot::from_env().cache_me();

    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("Error setting commands");
    bot
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

pub async fn debug_err(err: &anyhow::Error) {
    if let Ok(chat_id) = env::var(DEBUG_CHAT_ID_ENV_VAR)
        .unwrap_or("invalid".to_string())
        .parse::<i64>()
    {
        let _ = teloxide::Bot::from_env()
            .send_message(ChatId(chat_id), format!("{:#?}", err))
            .await;
    }
}
