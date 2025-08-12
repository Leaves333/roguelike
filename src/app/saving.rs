use color_eyre::{Result, eyre::Ok};
use std::{
    fs::File,
    io::{Read, Write},
};

use super::{App, Log, ObjectMap};
use crate::gamemap::GameMap;

impl App {
    /// saves current game state to a file
    pub fn save_game(&self) -> Result<()> {
        let save_data = serde_json::to_string(&(
            &self.gamemap,
            &self.objects,
            &self.inventory,
            &self.equipment,
            &self.log,
        ))?;

        let mut file = File::create("savegame")?;

        file.write_all(save_data.as_bytes())?;

        Ok(())
    }

    /// loads gamestate data from a save file
    /// NOTE: if the save file doesn't exist, it just crashes :sob:
    pub fn load_game(&mut self) -> Result<()> {
        let mut json_save_state = String::new();
        let mut file = File::open("savegame")?;
        file.read_to_string(&mut json_save_state)?;
        let result =
            serde_json::from_str::<(GameMap, ObjectMap, Vec<usize>, Vec<Option<usize>>, Log)>(
                &json_save_state,
            )?;

        self.gamemap = result.0;
        self.objects = result.1;
        self.inventory = result.2;
        self.equipment = result.3;
        self.log = result.4;

        Ok(())
    }
}
