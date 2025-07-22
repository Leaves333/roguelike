use render::GameScreen;

use crate::{
    entities::{self, spawn},
    gamemap::GameMap,
    procgen::generate_dungeon,
};

mod event_handler;
mod render;

pub const PLAYER: usize = 0;

pub struct App {
    gamemap: GameMap,
    game_screen: GameScreen,
    log: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        let player = spawn(0, 0, entities::player());
        let objects = vec![player];

        let log = Vec::new();

        let max_rooms = 30;
        let room_min_size = 6;
        let room_max_size = 10;
        let max_monsters_per_room = 1;

        let dungeon_width = 60;
        let dungeon_height = 16;

        let mut gamemap = generate_dungeon(
            max_rooms,
            room_min_size,
            room_max_size,
            max_monsters_per_room,
            dungeon_width,
            dungeon_height,
            objects,
        );

        gamemap.update_fov(8);
        Self {
            gamemap,
            game_screen: GameScreen::Main,
            log,
        }
    }
}
