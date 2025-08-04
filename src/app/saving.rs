use std::{fs::File, io::Write};

use color_eyre::{Result, eyre::Ok};

use super::App;

impl App {
    /// saves current game state to a file
    pub fn save_game(&self) -> Result<()> {
        // what needs to get serialized?

        // game_map
        // objects
        // next_id
        // inventory
        // log.

        let save_data = serde_json::to_string(&(
            &self.gamemap,
            &self.objects,
            &self.next_id,
            &self.inventory,
            &self.log,
        ))?;

        let mut file = File::create("savegame")?;

        file.write_all(save_data.as_bytes())?;

        Ok(())
    }
}
