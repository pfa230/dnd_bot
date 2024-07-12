use teloxide::{
    requests::{Requester, ResponseResult},
    types::Message,
    utils::command::BotCommands,
    Bot,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    Roll,
    Timers,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Roll => handle_roll(bot, msg).await,
        Command::Timers => handle_timers(bot, msg).await,
    }
}

async fn handle_roll(bot: Bot, msg: Message) -> ResponseResult<()> {
    log::info!("Roll called by {:?}", msg.from());

    bot.send_dice(msg.chat.id).await?;
    Ok(())
}

async fn handle_timers(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(
        msg.chat.id,
        format!("Not implemented yet, please come back later."),
    )
    .await?;
    Ok(())
}
