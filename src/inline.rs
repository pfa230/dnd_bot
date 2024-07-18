// use teloxide::{
//     requests::Requester,
//     types::{
//         InlineKeyboardButton, InlineKeyboardMarkup, InlineQuery,
//         InlineQueryResultArticle
//     },
// };
// use tracing::{info, instrument};

// use crate::utils::{Bot};

// #[instrument(skip(bot))]
// pub async fn handle_inline(
//     bot: Bot,
//     inline: InlineQuery,
// ) -> anyhow::Result<()> {
//     info!("Handling update");
//     // let dice1 = InlineQueryResultGif::new(
//     //     "0",
//     //     "https://dnd-bot-dice.s3.amazonaws.com/2dice.gif".parse()?,
//     //     "https://dnd-bot-dice.s3.amazonaws.com/1dice.gif".parse()?,
//     // ).gif_width(150).gif_height(150);
//     // let dice2 = InlineQueryResultGif::new(
//     //     "1",
//     //     "https://dnd-bot-dice.s3.amazonaws.com/2dice.gif".parse()?,
//     //     "https://dnd-bot-dice.s3.amazonaws.com/2dice.gif".parse()?,
//     // );
//     // let dice3 = InlineQueryResultGif::new(
//     //     "2",
//     //     "https://dnd-bot-dice.s3.amazonaws.com/3dice.gif".parse()?,
//     //     "https://dnd-bot-dice.s3.amazonaws.com/3dice.gif".parse()?,
//     // );
//     // info!("Sending {:?}", dice1);
//         let choose_debian_version = InlineQueryResultArticle::new(
//         "0",
//         "Chose debian version",
//         InputMessageContent::Text(InputMessageContentText::new("Debian versions:")),
//     )
//     .reply_markup(make_keyboard());

//     bot.answer_inline_query(inline.id, vec![choose_debian_version.into()]).await?;
//     // bot.answer_inline_query(inline.id, vec![dice1.into(), dice2.into(), dice3.into()])
//         // .await?;
//     Ok(())
// }

// fn make_keyboard() -> InlineKeyboardMarkup {
//     let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

//     let debian_versions = [
//         "Buzz", "Rex", "Bo", "Hamm", "Slink", "Potato", "Woody", "Sarge", "Etch", "Lenny",
//         "Squeeze", "Wheezy", "Jessie", "Stretch", "Buster", "Bullseye",
//     ];

//     for versions in debian_versions.chunks(3) {
//         let row = versions
//             .iter()
//             .map(|&version| InlineKeyboardButton::callback(version.to_owned(), version.to_owned()))
//             .collect();

//         keyboard.push(row);
//     }

//     InlineKeyboardMarkup::new(keyboard)
// }
