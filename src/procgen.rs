use rand::Rng;

use crate::gamemap::{GameMap, Tile, TileType};
use crate::los;

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

pub fn generate_dungeon(width: u16, height: u16) -> GameMap {
    let mut dungeon = GameMap::new(width, height);

    let room_one = RectangularRoom::new(20, 5, 10, 8);
    let room_two = RectangularRoom::new(35, 5, 10, 8);

    // fill two rooms
    for (x, y) in room_one.inner() {
        *dungeon.get_mut(x, y) = Tile::from_type(TileType::Floor);
    }
    for (x, y) in room_two.inner() {
        *dungeon.get_mut(x, y) = Tile::from_type(TileType::Floor);
    }

    // connect the rooms
    for (x, y) in tunnel_between(room_one.center(), room_two.center()) {
        *dungeon.get_mut(x, y) = Tile::from_type(TileType::Floor);
    }

    dungeon
}
