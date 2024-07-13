use teloxide::{
    requests::Requester,
    types::Message,
    utils::command::{BotCommands, ParseError},
};
use tracing::{info, instrument};

use crate::utils::{is_maxim, Bot, BotContext};

const MAX_DICE: usize = 5;

#[derive(BotCommands, PartialEq, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:",
    parse_with = "split"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "rolls the dice, takes optional number of dice.", parse_with=accepts_optional_number)]
    Roll(Option<usize>),
    #[command(description = "manage timers.")]
    Timers,
}

fn accepts_optional_number(input: String) -> Result<(Option<usize>,), ParseError> {
    match input.is_empty() {
        true => Ok((None,)),
        false => Ok((Some(
            input
                .parse::<usize>()
                .map_err(|e| ParseError::IncorrectFormat(e.into()))?,
        ),)),
    }
}

pub fn is_command(msg: &Message, context: &BotContext) -> Option<Command> {
    msg.text()
        .and_then(|text| Command::parse(text, &context.bot_name).ok())
}

#[instrument(skip(bot, context))]
pub async fn handle_command(
    bot: &Bot,
    msg: &Message,
    context: &BotContext,
    command: Command,
) -> anyhow::Result<()> {
    info!("Received command '{:?}'", command);

    match command {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Roll(num) => {
            let num_dice = num.unwrap_or(1);
            if num_dice > MAX_DICE {
                bot.send_message(
                    msg.chat.id,
                    format!("I could do only {MAX_DICE} dice at max, wanna try again?"),
                )
                .await?;
            } else {
                for _ in 0..num_dice {
                    match is_maxim(&msg) {
                        true => bot.send_sticker(msg.chat.id, context.dice_sticker.clone()).await?,
                        false => bot.send_dice(msg.chat.id).await?,
                    };
                }
            };
        }
        Command::Timers => {
            bot.send_message(
                msg.chat.id,
                format!("Not implemented yet, please come back later."),
            )
            .await?;
        }
    };
    Ok(())
}
