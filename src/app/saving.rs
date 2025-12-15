use color_eyre::{Result, eyre::Ok};
use serde::{Deserialize, Serialize};
use std::{
    collections::BinaryHeap,
    fs::File,
    io::{Read, Write},
};

use super::{App, Log, ObjectMap};
use crate::{app::Action, gamemap::GameMap};

#[derive(Serialize, Deserialize)]
struct SaveData {
    gamemap: GameMap,
    objects: ObjectMap,
    action_queue: BinaryHeap<Action>,
    inventory: Vec<usize>,
    equipment: Vec<Option<usize>>,
    log: Log,
}

impl App {
    /// saves current game state to a file
    pub fn save_game(&self) -> Result<()> {
        let save_data = SaveData {
            gamemap: self.gamemap.clone(),
            objects: self.objects.clone(),
            action_queue: self.action_queue.clone(),
            inventory: self.inventory.clone(),
            equipment: self.equipment.clone(),
            log: self.log.clone(),
        };

        let data_str = serde_json::to_string(&save_data)?;
        let mut file = File::create("savegame")?;
        file.write_all(data_str.as_bytes())?;
        Ok(())
    }

    /// loads gamestate data from a save file
    /// NOTE: if the save file doesn't exist, it just crashes :sob:
    pub fn load_game(&mut self) -> Result<()> {
        let mut save_string = String::new();
        let mut file = File::open("savegame")?;
        file.read_to_string(&mut save_string)?;
        let save_data = serde_json::from_str::<SaveData>(&save_string)?;

        self.gamemap = save_data.gamemap;
        self.objects = save_data.objects;
        self.action_queue = save_data.action_queue;
        self.inventory = save_data.inventory;
        self.equipment = save_data.equipment;
        self.log = save_data.log;

        Ok(())
    }
}
