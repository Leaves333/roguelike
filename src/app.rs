use std::collections::HashMap;

use render::GameScreen;

use crate::{
    components::Object,
    entities::{self, spawn},
    gamemap::GameMap,
};

mod event_handler;
mod procgen;
mod render;

pub const PLAYER: usize = 0;

pub struct App {
    gamemap: GameMap,
    game_screen: GameScreen,
    objects: HashMap<usize, Object>,
    next_id: usize,
    inventory: Vec<usize>,
    log: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        let player = spawn(0, 0, entities::player());
        let mut objects = HashMap::new();
        objects.insert(PLAYER, player);

        let next_id = 1;

        let max_rooms = 30;
        let room_min_size = 6;
        let room_max_size = 10;
        let max_monsters_per_room = 2;
        let max_items_per_room = 2;

        let dungeon_width = 60;
        let dungeon_height = 16;

        let mut app = Self {
            // NOTE: this is a dummy gamemap that will get
            // overriden by app.generate_dungeon() below
            gamemap: GameMap::new(0, 0, Vec::new()),

            // main is the default starting screen for the game
            game_screen: GameScreen::Main,
            objects,
            next_id,
            inventory: Vec::new(),
            log: Vec::new(),
        };

        app.generate_dungeon(
            max_rooms,
            room_min_size,
            room_max_size,
            max_monsters_per_room,
            max_items_per_room,
            dungeon_width,
            dungeon_height,
        );
        app.update_fov(8);

        app
    }
}
