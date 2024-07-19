use crate::tracker::Item;
use anyhow::bail;
use std::str::FromStr;
use strum::{AsRefStr, EnumString};
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};

#[derive(Clone, EnumString, AsRefStr)]
pub enum CallbackAction {
    TickTimer,
    DeleteTimer,
    AddHarm,
    SubHarm,
    DeleteHarm,
    AddStress,
    SubStress,
    DeleteStress,
}

pub struct Callback {
    pub id: usize,
    pub chat_id: ChatId,
    pub action: CallbackAction,
}

impl CallbackAction {
    fn button_name(&self, item_name: &str) -> String {
        match self {
            CallbackAction::TickTimer => format!("{} -1", item_name),
            CallbackAction::DeleteTimer => format!("Delete {}", item_name),
            CallbackAction::AddHarm => format!("{} +1", item_name),
            CallbackAction::SubHarm => format!("{} -1", item_name),
            CallbackAction::DeleteHarm => format!("Delete {}", item_name),
            CallbackAction::AddStress => format!("{} +1", item_name),
            CallbackAction::SubStress => format!("{} -1", item_name),
            CallbackAction::DeleteStress => format!("Delete {}", item_name),
        }
    }
}

impl Callback {
    fn serialize(&self) -> String {
        format!(
            "{}|{}|{}",
            self.id,
            self.chat_id,
            self.action.as_ref()
        )
    }

    pub fn deserialize(cb: &str) -> anyhow::Result<Self> {
        let split = cb.split("|").collect::<Vec<_>>();
        if split.len() != 3 {
            bail!("Invalid callback data: {}", cb);
        }
        Ok(Callback {
            id: split[0].parse()?,
            chat_id: ChatId(split[1].parse()?),
            action: CallbackAction::from_str(split[2])?,
        })
    }
}

pub fn make_timers_keyboard(chat_id: ChatId, timers: &[Item]) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for timer in timers {
        keyboard.push(vec![
            create_button(chat_id, &timer, CallbackAction::TickTimer),
            create_button(chat_id, &timer, CallbackAction::DeleteTimer),
        ]);
    }

    InlineKeyboardMarkup::new(keyboard)
}

pub fn make_harm_keyboard(chat_id: ChatId, items: &[Item]) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for item in items {
        keyboard.push(vec![
            create_button(chat_id, &item, CallbackAction::AddHarm),
            create_button(chat_id, &item, CallbackAction::SubHarm),
            create_button(chat_id, &item, CallbackAction::DeleteHarm),
        ]);
    }

    InlineKeyboardMarkup::new(keyboard)
}

pub fn make_stress_keyboard(chat_id: ChatId, items: &[Item]) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for item in items {
        keyboard.push(vec![
            create_button(chat_id, &item, CallbackAction::AddStress),
            create_button(chat_id, &item, CallbackAction::SubStress),
            create_button(chat_id, &item, CallbackAction::DeleteStress),
        ]);
    }

    InlineKeyboardMarkup::new(keyboard)
}

fn create_button(chat_id: ChatId, item: &Item, action: CallbackAction) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(
        action.button_name(&item.name),
        Callback {
            id: item.id,
            chat_id,
            action,
        }
        .serialize(),
    )
}
