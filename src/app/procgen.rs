use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;

use crate::app::{App, PLAYER};
use crate::components::Object;
use crate::entities::spawn;
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
    room_min_size: u16,
    room_max_size: u16,
    width: u16,
    height: u16,
    level: u16,
}

impl DungeonConfig {
    // default dungeon config. starts at level 1.
    pub fn default() -> Self {
        Self {
            max_rooms: 30,
            room_min_size: 6,
            room_max_size: 10,
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
    let troll_weight = from_dungeon_level(
        &[
            Transition {
                level: 2,
                value: 15,
            },
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
        ],
        level,
    );

    vec![(entities::orc, orc_weight), (entities::troll, troll_weight)]
}

fn item_table(level: u16) -> Vec<(fn() -> Object, usize)> {
    let potion_weight = 35;
    let lightning_weight = from_dungeon_level(
        &[Transition {
            level: 2,
            value: 15,
        }],
        level,
    );
    let weapon_weight = 10;

    vec![
        (entities::potion_cure_wounds, potion_weight),
        (entities::scroll_lightning, lightning_weight),
        (entities::weapon_dagger, weapon_weight),
    ]
}

impl App {
    /// replaces the current gamemap for the app with a new one
    pub fn generate_dungeon(&mut self, config: DungeonConfig) {
        let mut dungeon = GameMap::new(config.width, config.height, config.level, Vec::new());
        let mut rooms: Vec<RectangularRoom> = Vec::new();
        dungeon.object_ids.push(PLAYER); // player is currently in this gamemap

        let mut rng = rand::rng();
        for _ in 0..config.max_rooms {
            let room_width = rng.random_range(config.room_min_size..=config.room_max_size);
            let room_height = rng.random_range(config.room_min_size..=config.room_max_size);

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
                *dungeon.get_mut(x, y) = Tile::from_type(TileType::Floor);
            }

            if rooms.is_empty() {
                // player starts in the first room
                let position = &mut self.objects.get_mut(&PLAYER).unwrap().pos;
                (position.x, position.y) = new_room.center();
            } else {
                // dig tunnel between current room and previous
                for (x, y) in tunnel_between(rooms.last().unwrap().center(), new_room.center()) {
                    *dungeon.get_mut(x, y) = Tile::from_type(TileType::Floor);
                }
            }

            // loot tables for monsters and items
            let max_monsters_table = &[
                Transition { level: 1, value: 2 },
                Transition { level: 4, value: 3 },
                Transition { level: 6, value: 4 },
            ];
            let max_monsters = from_dungeon_level(max_monsters_table, dungeon.level);

            let max_items_table = &[
                Transition { level: 1, value: 1 },
                Transition { level: 3, value: 2 },
            ];
            let max_items = from_dungeon_level(max_items_table, dungeon.level);

            let monsters = monster_table(dungeon.level);
            let items = item_table(dungeon.level);
            self.place_objects(&new_room, &mut dungeon, &monsters, max_monsters);
            self.place_objects(&new_room, &mut dungeon, &items, max_items);

            rooms.push(new_room);
        }

        let last_room = rooms.last().unwrap();
        let (stairs_x, stairs_y) = last_room.center();
        let id = self
            .objects
            .add(spawn(stairs_x, stairs_y, entities::stairs()));
        dungeon.object_ids.push(id);

        self.gamemap = dungeon;
    }

    fn place_objects(
        &mut self,
        room: &RectangularRoom,
        dungeon: &mut GameMap,
        object_weights: &Vec<(fn() -> Object, usize)>,
        maximum_objects: usize,
    ) {
        let mut rng = rand::rng();
        let dist = WeightedIndex::new(object_weights.iter().map(|x| x.1)).unwrap();

        let number_of_items = rng.random_range(0..=maximum_objects);
        for _ in 0..number_of_items {
            let x = rng.random_range((room.x1 + 1)..room.x2);
            let y = rng.random_range((room.y1 + 1)..room.y2);

            // check if it intersects with any entities
            match self.get_blocking_object_id(x, y) {
                Some(_) => {
                    continue;
                }
                None => {}
            }

            let entity_callback = object_weights[dist.sample(&mut rng)].0;
            dungeon
                .object_ids
                .push(self.objects.add(spawn(x, y, entity_callback())));
        }
    }
}
