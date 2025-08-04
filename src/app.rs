use std::collections::HashMap;

use ratatui::style::Style;

use crate::{
    components::{Object, Position},
    engine::TargetingMode,
    entities::{self, spawn},
    gamemap::GameMap,
};

mod event_handler;
mod procgen;
mod render;

pub const PLAYER: usize = 0;

pub struct Log {
    messages: Vec<(String, Style)>,
}

impl Log {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    /// add the new message as a tuple, with the text and the style
    pub fn add<T: Into<String>, U: Into<Style>>(&mut self, message: T, style: U) {
        self.messages.push((message.into(), style.into()));
    }

    /// create a `DoubleEndedIterator` over the messages
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &(String, Style)> {
        self.messages.iter()
    }

    /// create a `DoubleEndedIterator` over the messages
    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

pub struct App {
    pub gamemap: GameMap,
    pub game_screen: GameScreen,
    pub objects: HashMap<usize, Object>,
    pub next_id: usize,
    pub inventory: Vec<usize>,
    pub log: Log,
}

pub enum GameScreen {
    /// the main menu
    Menu,
    /// default gameplay screen, with world map and log
    Main,
    /// display fullscreen log with offset
    Log { offset: usize },
    /// use the examine cursor to look at tiles
    Examine { cursor: Position },
    /// mode for aiming targetable skills at enemies
    Targeting {
        cursor: Position,
        targeting: TargetingMode,
        text: String,
        inventory_idx: usize,
    },
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

        let dungeon_width = 80;
        let dungeon_height = 24;

        let mut app = Self {
            // NOTE: this is a dummy gamemap that will get
            // overriden by app.generate_dungeon() below
            gamemap: GameMap::new(0, 0, Vec::new()),

            game_screen: GameScreen::Menu, // start the game on the main menu
            objects,
            next_id,
            inventory: Vec::new(),
            log: Log::new(),
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
