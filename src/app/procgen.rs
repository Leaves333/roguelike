use rand::{Rng, random_ratio};

use crate::app::{App, PLAYER};
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

impl App {
    /// replaces the current gamemap for the app with a new one
    pub fn generate_dungeon(
        &mut self,
        max_rooms: u16,
        room_min_size: u16,
        room_max_size: u16,
        max_monsters_per_room: u16,
        max_items_per_room: u16,
        width: u16,
        height: u16,
    ) {
        let mut dungeon = GameMap::new(width, height, Vec::new());
        let mut rooms: Vec<RectangularRoom> = Vec::new();
        dungeon.object_ids.push(PLAYER); // player is currently in this gamemap

        let mut rng = rand::rng();
        for _ in 0..max_rooms {
            let room_width = rng.random_range(room_min_size..=room_max_size);
            let room_height = rng.random_range(room_min_size..=room_max_size);

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

            self.place_objects(
                &new_room,
                &mut dungeon,
                max_monsters_per_room,
                max_items_per_room,
            );

            rooms.push(new_room);
        }

        self.gamemap = dungeon;
    }

    fn place_objects(
        &mut self,
        room: &RectangularRoom,
        dungeon: &mut GameMap,
        maximum_monsters: u16,
        maximum_items: u16,
    ) {
        let mut rng = rand::rng();

        // place monsters
        let number_of_monsters = rng.random_range(0..=maximum_monsters);
        for _ in 0..number_of_monsters {
            let x = rng.random_range((room.x1 + 1)..room.x2);
            let y = rng.random_range((room.y1 + 1)..room.y2);

            // check if it intersects with any entities
            match self.get_blocking_object_id(x, y) {
                Some(_) => {
                    continue;
                }
                None => {}
            }

            if random_ratio(4, 5) {
                self.objects
                    .insert(self.next_id, spawn(x, y, entities::orc()));
            } else {
                self.objects
                    .insert(self.next_id, spawn(x, y, entities::troll()));
            }
            dungeon.object_ids.push(self.next_id);
            self.next_id += 1;
        }

        // use similar logic for items
        let number_of_items = rng.random_range(0..=maximum_items);
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

            let mut rng = rand::rng();
            let dice = rng.random::<f32>();

            if dice > 0.5 {
                self.objects
                    .insert(self.next_id, spawn(x, y, entities::scroll_lightning()));
            } else {
                self.objects
                    .insert(self.next_id, spawn(x, y, entities::potion_cure_wounds()));
            }

            dungeon.object_ids.push(self.next_id);
            self.next_id += 1;
        }
    }
}
