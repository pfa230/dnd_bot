use teloxide::{requests::Requester, types::Message, utils::command::BotCommands};
use tracing::debug;

use crate::utils::{Bot, Context};

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:",
    parse_with = "split"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "rolle the dice.")]
    Roll,
    #[command(description = "manage timers.")]
    Timers,
}

pub fn is_command(msg: &Message, context: &Context) -> Option<Command> {
    let bot_name = context
        .me
        .username
        .as_ref()
        .expect("Bots must have a username");

    msg.text()
        .and_then(|text| Command::parse(text, &bot_name).ok())
}

pub async fn handle_command(bot: &Bot, msg: &Message, command: Command) -> anyhow::Result<()> {
    debug!("Got command {:?}", command);
    match command {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Roll => bot.send_dice(msg.chat.id).await?,
        Command::Timers => {
            bot.send_message(
                msg.chat.id,
                format!("Not implemented yet, please come back later."),
            )
            .await?
        }
    };
    Ok(())
}
