use anyhow::{anyhow, bail};
use serde::{Deserialize, Serialize};
use teloxide::types::MessageId;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timer {
    // Should be first for sorting purposes
    pub name: String,
    pub id: usize,
    pub value: i32,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Player {
    // Should be first for sorting purposes
    pub name: String,
    pub id: usize,
    pub harm: i32,
    pub stress: i32,
}

#[derive(Serialize, Deserialize)]
struct Timers(Vec<Timer>);
#[derive(Serialize, Deserialize)]
struct Players(Vec<Player>);

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Tracker {
    pub timers: Vec<Timer>,
    pub players: Vec<Player>,
    pub timers_msg: Option<TimersMsg>,
    pub players_msg: Option<PlayersMsg>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimersMsg {
    pub msg_id: MessageId,
    pub kb_id: MessageId,
    pub keyboard_active: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PlayersKeyboard {
    Harm,
    Stress,
    ManagePlayers,
    None,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayersMsg {
    pub msg_id: MessageId,
    pub kb_id: MessageId,
    pub active_keyboard: PlayersKeyboard,
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            ..Default::default()
        }
    }

    pub fn create_timer(&mut self, name: &str, start_value: i32) -> anyhow::Result<Timer> {
        if self
            .timers
            .iter()
            .find(|timer| timer.name.eq(name))
            .is_some()
        {
            bail!("Timer {} already present", name);
        }
        let next_id = self
            .timers
            .iter()
            .map(|timer| timer.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .unwrap();

        let timer = Timer {
            id: next_id,
            name: name.to_owned(),
            value: start_value,
        };
        self.timers.push(timer.clone());
        self.timers.sort();
        Ok(timer)
    }

    pub fn create_player(&mut self, name: &str) -> anyhow::Result<Player> {
        if self
            .players
            .iter()
            .find(|player| player.name.eq(name))
            .is_some()
        {
            bail!("Player {} already present", name);
        }
        let next_id = self
            .players
            .iter()
            .map(|player| player.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .unwrap();

        let player = Player {
            id: next_id,
            name: name.to_owned(),
            harm: 0,
            stress: 0,
        };
        self.players.push(player.clone());
        self.players.sort();
        Ok(player)
    }

    pub fn get_timer(&mut self, id: usize) -> anyhow::Result<&mut Timer> {
        self.timers.iter_mut().find(|timer| timer.id == id).ok_or(anyhow!("Timer id {} not found", id))
    }

    pub fn get_player(&mut self, id: usize) -> anyhow::Result<&mut Player> {
        self.players.iter_mut().find(|player| player.id == id).ok_or(anyhow!("Player id {} not found", id))
    }

    pub fn change_timer(&mut self, id: usize, val: i32) -> anyhow::Result<Timer> {
        let timer = self.get_timer(id)?;
        timer.value = timer.value.checked_add(val).expect("Overflow");
        Ok(timer.clone())
    }

    pub fn change_harm(&mut self, id: usize, val: i32) -> anyhow::Result<Player> {
        let player = self.get_player(id)?;
        player.harm = player.harm.checked_add(val).expect("Overflow");
        Ok(player.clone())
    }

    pub fn change_stress(&mut self, id: usize, val: i32) -> anyhow::Result<Player> {
        let player = self.get_player(id)?;
        player.stress = player.stress.checked_add(val).expect("Overflow");
        Ok(player.clone())
    }

    pub fn delete_timer(&mut self, id: usize) -> anyhow::Result<Timer> {
        let pos = self
            .timers
            .iter()
            .position(|timer| timer.id == id)
            .ok_or(anyhow!("Timer id {} not found", id));
        pos.and_then(|pos| Ok(self.timers.remove(pos)))
    }

    pub fn delete_player(&mut self, id: usize) -> anyhow::Result<Player> {
        let pos = self
            .players
            .iter()
            .position(|player| player.id == id)
            .ok_or(anyhow!("Player id {} not found", id));
        pos.and_then(|pos| Ok(self.players.remove(pos)))
    }
}
