use teloxide::{requests::Requester, types::Message, utils::command::BotCommands};
use tracing::{info, instrument};

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
    msg.text()
        .and_then(|text| Command::parse(text, &context.bot_name).ok())
}

#[instrument(skip(bot))]
pub async fn handle_command(bot: &Bot, msg: &Message, command: Command) -> anyhow::Result<()> {
    info!("Received command '{:?}'", command);

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
