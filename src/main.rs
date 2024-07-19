use std::env;

use ::tracing::{info, instrument};
use anyhow::anyhow;
use dispatcher::dispatch_update;
use dotenv::dotenv;
use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestPayloadExt, Response};
use teloxide::prelude::*;
use utils::{authorize, error_response, init_bot, success_response, Bot};

mod callback;
mod context;
mod dispatcher;
mod handler;
mod inline;
mod tracker;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match env::var("ON_LAMBDA").as_deref() {
        Ok("1") => run_on_lambda(init_bot().await).await,
        _ => {
            dotenv().ok();
            run_locally(init_bot().await).await
        }
    }
}

#[instrument(skip(bot))]
async fn run_on_lambda(bot: Bot) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .with_current_span(false)
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();

    info!("Starting serverless bot...");

    run(service_fn(|req| handle_lambda_request(bot.clone(), req)))
        .await
        .map_err(|err| anyhow!("{:?}", err))
}

#[instrument(skip(bot, request))]
async fn handle_lambda_request(bot: Bot, request: Request) -> Result<Response<Body>, Error> {
    if let Err(e) = authorize(&request) {
        return error_response(401, format!("Unauthorized: {e}"));
    }

    let update: Update = match request.payload() {
        Ok(Some(payload)) => payload,
        Ok(None) => {
            return error_response(400, "Empty payload");
        }
        Err(e) => {
            return error_response(400, format!("Invalid payload: {e}"));
        }
    };

    match dispatch_update(bot, update).await {
        Ok(()) => success_response(),
        Err(e) => error_response(400, format!("Error: {e}")),
    }
}

#[instrument(skip(bot))]
async fn run_locally(bot: Bot) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();

    info!("Starting local bot...");

    Dispatcher::builder(bot, dptree::endpoint(dispatch_update))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
