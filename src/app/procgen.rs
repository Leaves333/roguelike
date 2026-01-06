use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use ratatui::style::Color;

use crate::app::{Action, App, PLAYER};
use crate::components::Object;
use crate::engine::get_blocking_object_id;
use crate::gamemap::{GameMap, Tile, TileType};
use crate::{entities, los};

struct RectangularRoom {
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
}

impl RectangularRoom {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x1: x,
            y1: y,
            x2: x + width,
            y2: y + height,
        }
    }

    pub fn center(&self) -> (u16, u16) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    // returns an iterator over all the cells on the inside of the room
    pub fn inner(&self) -> impl Iterator<Item = (u16, u16)> {
        (self.y1 + 1..self.y2).flat_map(move |y| (self.x1 + 1..self.x2).map(move |x| (x, y)))
    }

    // checks if this room intersects with another room
    pub fn intersects(&self, other: &RectangularRoom) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }
}

pub fn tunnel_between(start: (u16, u16), end: (u16, u16)) -> Vec<(u16, u16)> {
    // returns an L-shaped tunnel between these two points

    let (x1, y1) = (start.0 as i32, start.1 as i32);
    let (x2, y2) = (end.0 as i32, end.1 as i32);

    let mut rng = rand::rng();
    let (corner_x, corner_y) = { if rng.random() { (x2, y1) } else { (x1, y2) } };

    let seg_one: Vec<(u16, u16)> = los::bresenham((x1, y1), (corner_x, corner_y))
        .iter()
        .map(|&(x, y)| (x as u16, y as u16))
        .collect();
    let seg_two: Vec<(u16, u16)> = los::bresenham((corner_x, corner_y), (x2, y2))
        .iter()
        .map(|&(x, y)| (x as u16, y as u16))
        .collect();
    [seg_one, seg_two].concat()
}

pub struct DungeonConfig {
    max_rooms: u16,
    room_min_width: u16,
    room_max_width: u16,
    room_min_height: u16,
    room_max_height: u16,
    width: u16,
    height: u16,
    level: u16,
}

impl DungeonConfig {
    // default dungeon config. starts at level 1.
    pub fn default() -> Self {
        Self {
            max_rooms: 200,
            room_min_width: 7,
            room_max_width: 25,
            room_min_height: 4,
            room_max_height: 7,
            width: 80,
            height: 24,
            level: 1,
        }
    }

    pub fn set_level(mut self, level: u16) -> Self {
        self.level = level;
        self
    }
}

struct Transition {
    level: u16,
    value: usize,
}

/// returns a value that depends on level. table specifies what
/// value occurs after each level, default is 0. assumes that transitions
/// are sorted by level in the table slice
fn from_dungeon_level(table: &[Transition], level: u16) -> usize {
    table
        .iter()
        .rev()
        .find(|transition| level >= transition.level)
        .map_or(0, |transition| transition.value)
}

fn monster_table(level: u16) -> Vec<(fn() -> Object, usize)> {
    let orc_weight = 80;
    const RAT_WEIGHT_TABLE: &[Transition; 3] = &[
        Transition {
            level: 2,
            value: 30,
        },
        Transition {
            level: 3,
            value: 50,
        },
        Transition {
            level: 5,
            value: 70,
        },
    ];
    let rat_weight = from_dungeon_level(RAT_WEIGHT_TABLE, level);

    const TROLL_WEIGHT_TABLE: &[Transition; 3] = &[
        Transition {
            level: 3,
            value: 30,
        },
        Transition {
            level: 5,
            value: 45,
        },
        Transition {
            level: 7,
            value: 60,
        },
    ];
    let troll_weight = from_dungeon_level(TROLL_WEIGHT_TABLE, level);

    vec![
        (entities::orc, orc_weight),
        (entities::rat, rat_weight),
        (entities::troll, troll_weight),
    ]
}

fn item_table(level: u16) -> Vec<(fn() -> Object, usize)> {
    let potion_weight = 30;

    let lightning_weight = from_dungeon_level(
        &[Transition {
            level: 2,
            value: 15,
        }],
        level,
    );

    let dagger_weight = 5;
    let longsword_weight = from_dungeon_level(&[Transition { level: 4, value: 5 }], level);
    let helmet_weight = from_dungeon_level(&[Transition { level: 3, value: 5 }], level);
    let leather_weight = from_dungeon_level(&[Transition { level: 2, value: 5 }], level);
    let plate_weight = from_dungeon_level(&[Transition { level: 5, value: 5 }], level);

    vec![
        (entities::potion_cure_wounds, potion_weight),
        (entities::scroll_lightning, lightning_weight),
        (entities::weapon_dagger, dagger_weight),
        (entities::weapon_longsword, longsword_weight),
        (entities::helmet, helmet_weight),
        (entities::leather_armor, leather_weight),
        (entities::plate_armor, plate_weight),
    ]
}

const MAX_MONSTERS_TABLE: &[Transition; 3] = &[
    Transition { level: 1, value: 2 },
    Transition { level: 4, value: 3 },
    Transition { level: 6, value: 4 },
];

const MAX_ITEMS_TABLE: &[Transition; 2] = &[
    Transition { level: 1, value: 1 },
    Transition { level: 3, value: 2 },
];

impl App {
    /// replaces the current gamemap for the app with a new one
    pub fn generate_dungeon(&mut self, config: DungeonConfig) {
        let mut dungeon = GameMap::new(config.width, config.height, config.level, Vec::new());
        let mut rooms: Vec<RectangularRoom> = Vec::new();
        dungeon.object_ids.push(PLAYER); // player is currently in this gamemap

        let mut rng = rand::rng();
        for _ in 0..config.max_rooms {
            let room_width = rng.random_range(config.room_min_width..=config.room_max_width);
            let room_height = rng.random_range(config.room_min_height..=config.room_max_height);

            let x = rng.random_range(0..dungeon.width - room_width);
            let y = rng.random_range(0..dungeon.height - room_height);

            let new_room = RectangularRoom::new(x, y, room_width, room_height);

            // break if the new room intersects with a previous room
            let has_intersection = rooms
                .iter()
                .fold(false, |b, room| b || room.intersects(&new_room));
            if has_intersection {
                continue;
            }

            // dig out the room's inner area
            for (x, y) in new_room.inner() {
                *dungeon.get_mut(x, y) = Tile::new(TileType::Floor);
            }

            if !rooms.is_empty() {
                // dig tunnel between current room and previous
                for (x, y) in tunnel_between(rooms.last().unwrap().center(), new_room.center()) {
                    *dungeon.get_mut(x, y) = Tile::new(TileType::Floor);
                }
            }

            rooms.push(new_room);
        }

        // generate contents in rooms
        for room in &rooms {
            // loot tables for monsters and items
            let max_monsters = from_dungeon_level(MAX_MONSTERS_TABLE, dungeon.level);
            let max_items = from_dungeon_level(MAX_ITEMS_TABLE, dungeon.level);

            let monsters = monster_table(dungeon.level);
            let items = item_table(dungeon.level);

            // panic!(
            //     "height: {}, width: {}, len: {}",
            //     dungeon.height,
            //     dungeon.width,
            //     dungeon.tiles.len()
            // );

            // add these items to the gamemap
            self.place_objects(&room, &mut dungeon, &monsters, max_monsters, false);
            self.place_objects(&room, &mut dungeon, &items, max_items, true);
        }

        // spawn player in the center of the first room
        let first_room = rooms.first().unwrap();
        let (player_x, player_y) = first_room.center();
        dungeon.place_blocker(PLAYER, player_x, player_y);

        // spawn the stairs in the center of the last room
        let last_room = rooms.last().unwrap();
        let (stairs_x, stairs_y) = last_room.center();
        let stairs_id = self.objects.add(entities::stairs());
        dungeon.place_item(stairs_id, stairs_x, stairs_y);
        dungeon.object_ids.push(stairs_id);

        self.gamemap = dungeon;
    }

    fn place_objects(
        &mut self,
        room: &RectangularRoom,
        dungeon: &mut GameMap,
        object_weights: &Vec<(fn() -> Object, usize)>,
        maximum_objects: usize,
        is_item: bool,
    ) {
        let mut rng = rand::rng();
        let dist = WeightedIndex::new(object_weights.iter().map(|x| x.1)).unwrap();

        let number_of_items = rng.random_range(0..=maximum_objects);
        for _ in 0..number_of_items {
            let x = rng.random_range((room.x1 + 1)..room.x2);
            let y = rng.random_range((room.y1 + 1)..room.y2);

            // panic!(
            //     "height: {}, width: {}, len: {}",
            //     dungeon.height,
            //     dungeon.width,
            //     dungeon.tiles.len()
            // );

            // check if it intersects with any entities
            let tile = dungeon.get_ref(x, y);
            if is_item {
                match tile.item {
                    Some(_) => {
                        continue;
                    }
                    None => {}
                }
            } else {
                match tile.blocker {
                    Some(_) => {
                        continue;
                    }
                    None => {}
                }
            }

            // randomly select which object to spawn
            let entity_callback = object_weights[dist.sample(&mut rng)].0;

            let object = entity_callback();
            let has_ai = object.ai.is_some();
            let object_id = self.objects.add(object);

            if is_item {
                dungeon.place_item(object_id, x, y);
            } else {
                dungeon.place_blocker(object_id, x, y);
            }

            // objects with an AI component should be added into the action queue
            if has_ai {
                self.action_queue.push(Action {
                    // NOTE: 100 is magic number to ensure monsters don't double act on the first turn
                    // ideally we should add something to the effect of how long it takes the
                    // monster to take an action
                    time: self.time + 100,
                    id: object_id,
                });
                self.add_to_log(
                    format!("added action for object {object_id}"),
                    Color::default(),
                );
            }

            // let this floor of the dungeon own the object
            dungeon.object_ids.push(object_id);
        }
    }
}
