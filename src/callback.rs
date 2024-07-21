use anyhow::bail;
use std::str::FromStr;
use strum::{AsRefStr, EnumString};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::tracker::{Player, Timer};

#[derive(Clone, EnumString, AsRefStr)]
pub enum CallbackAction {
    NoAction,
    AddTimer,
    SubTimer,
    DeleteTimer,
    AddHarm,
    SubHarm,
    AddStress,
    SubStress,
    DeletePlayer,
    ShowTimersKb,
    ShowPlayersKb,
    ShowHarmKb,
    ShowStressKb,
    HideTimersKb,
    HidePlayersKb,
}

pub struct Callback {
    pub item_id: usize,
    pub action: CallbackAction,
}

impl Callback {
    fn serialize(&self) -> String {
        format!("{}|{}", self.item_id, self.action.as_ref())
    }

    pub fn deserialize(cb: &str) -> anyhow::Result<Self> {
        let split = cb.split("|").collect::<Vec<_>>();
        if split.len() != 2 {
            bail!("Invalid callback data: {}", cb);
        }
        Ok(Callback {
            item_id: split[0].parse()?,
            action: CallbackAction::from_str(split[1])?,
        })
    }
}

pub fn make_manage_timers_keyboard(timers: &[Timer]) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for timer in timers.iter() {
        keyboard.push(vec![
            create_button(timer.id, timer.name.as_str(), CallbackAction::NoAction),
            create_button(timer.id, "+1", CallbackAction::AddTimer),
            create_button(timer.id, "-1", CallbackAction::SubTimer),
            create_button(timer.id, "Delete", CallbackAction::DeleteTimer),
        ]);
    }
    // keyboard.push(vec![create_button(0, "Hide", CallbackAction::HideTimersKb)]);

    InlineKeyboardMarkup::new(keyboard)
}

pub fn make_manage_harm_keyboard(players: &[Player]) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for player in players.iter() {
        keyboard.push(vec![
            create_button(player.id, player.name.as_str(), CallbackAction::NoAction),
            create_button(player.id, "+1 harm", CallbackAction::AddHarm),
            create_button(player.id, "-1 harm", CallbackAction::SubHarm),
        ]);
    }
    keyboard.push(vec![create_button(0, "Back", CallbackAction::HidePlayersKb)]);

    InlineKeyboardMarkup::new(keyboard)
}

pub fn make_manage_stress_keyboard(players: &[Player]) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for player in players.iter() {
        keyboard.push(vec![
            create_button(player.id, player.name.as_str(), CallbackAction::NoAction),
            create_button(player.id, "+1 stress", CallbackAction::AddStress),
            create_button(player.id, "-1 stress", CallbackAction::SubStress),
        ]);
    }
    keyboard.push(vec![create_button(0, "Back", CallbackAction::HidePlayersKb)]);

    InlineKeyboardMarkup::new(keyboard)
}

pub fn make_manage_players_keyboard(players: &[Player]) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for player in players.iter() {
        keyboard.push(vec![
            create_button(player.id, player.name.as_str(), CallbackAction::NoAction),
            create_button(player.id, "Delete", CallbackAction::DeletePlayer),
        ]);
    }
    keyboard.push(vec![create_button(
        0,
        "Back",
        CallbackAction::HidePlayersKb,
    )]);

    InlineKeyboardMarkup::new(keyboard)
}

pub fn make_players_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    keyboard.push(vec![
        create_button(0, "Manage harm", CallbackAction::ShowHarmKb),
        create_button(0, "Manage stress", CallbackAction::ShowStressKb),
        create_button(0, "Manage players", CallbackAction::ShowPlayersKb),
    ]);

    InlineKeyboardMarkup::new(keyboard)
}

pub fn make_timers_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    keyboard.push(vec![create_button(
        0,
        "Manage",
        CallbackAction::ShowTimersKb,
    )]);

    InlineKeyboardMarkup::new(keyboard)
}

fn create_button(item_id: usize, name: &str, action: CallbackAction) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(name, Callback { item_id, action }.serialize())
}
