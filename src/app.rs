use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    usize,
};

use ratatui::style::Style;
use serde::{Deserialize, Serialize};

use crate::{
    components::{Object, Position, SLOT_ORDERING},
    engine::TargetingMode,
    entities::{self, spawn},
    gamemap::GameMap,
};

mod event_handler;
pub(crate) mod procgen;
mod render;
mod saving;

pub const PLAYER: usize = 0;
pub const VIEW_RADIUS: u16 = 8;
pub const INVENTORY_SIZE: usize = 10;

#[derive(Serialize, Deserialize, Clone)]
pub struct LogEntry {
    time: u64,
    message: String,
    style: Style,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Log {
    messages: Vec<LogEntry>,
}

impl Log {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    /// create a `DoubleEndedIterator` over the messages
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &LogEntry> {
        self.messages.iter()
    }

    /// return the number of messages in the log
    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectMap {
    objects: HashMap<usize, Object>,
    next_id: usize,
}

impl ObjectMap {
    /// constructs a new ObjectMap and inserts the player object into it.
    /// this guarantees that player id is always 0.
    pub fn new(player: Object) -> Self {
        let mut map = Self {
            objects: HashMap::new(),
            next_id: 0,
        };
        map.add(player);
        map
    }

    /// add a new object into the map, incrementing the next id
    /// returns the id that the object was allocated
    pub fn add(&mut self, obj: Object) -> usize {
        let ret = self.next_id;
        self.objects.insert(self.next_id, obj);
        self.next_id += 1;
        ret
    }

    pub fn get(&self, id: &usize) -> Option<&Object> {
        self.objects.get(id)
    }

    pub fn get_mut(&mut self, id: &usize) -> Option<&mut Object> {
        self.objects.get_mut(id)
    }

    /// returns a mutable reference to the underlying hashmap.
    /// WARN: do not add items into the hashmap using this method!
    ///       it will not update next_id
    pub fn get_contents(&mut self) -> &mut HashMap<usize, Object> {
        &mut self.objects
    }

    #[allow(dead_code)]
    pub fn next_id(&self) -> usize {
        self.next_id
    }
}

/// struct for the priority queue that decides which object id should act next
#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub time: u64, // time that the action should be performed
    pub id: usize, // id of object that should take an action
}

impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max heap
        // so we reverse the comparison for the time
        // id doesn't really matter since its just a tiebreak
        return other
            .time
            .cmp(&self.time)
            .then_with(|| self.id.cmp(&other.id));
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct App {
    pub gamemap: GameMap,
    pub game_screen: GameScreen,
    pub objects: ObjectMap,
    pub action_queue: BinaryHeap<Action>,
    pub time: u64,
    pub inventory: Vec<usize>,
    pub equipment: Vec<Option<usize>>,
    pub log: Log,
}

/// a singleton enum describing the current screen to display
#[derive(PartialEq, Eq)]
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
        let objects = ObjectMap::new(player);

        Self {
            // NOTE: this is a dummy gamemap that should get overwritten when
            // loading or creating a new game
            gamemap: GameMap::new(0, 0, 0, Vec::new()),

            game_screen: GameScreen::Menu, // start the game on the main menu
            objects,
            action_queue: BinaryHeap::new(),
            time: 0,
            inventory: Vec::new(),
            equipment: vec![None; SLOT_ORDERING.len()],
            log: Log::new(),
        }
    }

    /// add the new message as a tuple, with the text and the style
    pub fn add_to_log<T: Into<String>, U: Into<Style>>(&mut self, message: T, style: U) {
        let entry = LogEntry {
            time: self.time,
            message: message.into(),
            style: style.into(),
        };
        self.log.messages.push(entry);
    }
}
