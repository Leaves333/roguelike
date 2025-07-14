use rand::Rng;

use crate::gamemap::{GameMap, Tile, TileType};

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

pub fn tunnel_between(start: (u16, u16), end: (u16, u16)) -> impl Iterator<Item = (u16, u16)> {
    // returns an L-shaped tunnel between these two points

    let (x1, y1) = start;
    let (x2, y2) = start;

    // TODO: implement bresenham's algo for line of sight

    let (mut corner_x, mut corner_y) = (0, 0);
    let mut rng = rand::rng();
    if rng.random() {
        (corner_x, corner_y) = (x2, y1);
    } else {
        (corner_x, corner_y) = (x1, y2);
    }

    (y1 + 1..y2).flat_map(move |y| (x1 + 1..x2).map(move |x| (x, y)))
}

pub fn generate_dungeon(width: u16, height: u16) -> GameMap {
    let mut dungeon = GameMap::new(width, height);

    let room_one = RectangularRoom::new(20, 5, 10, 8);
    let room_two = RectangularRoom::new(35, 5, 10, 8);

    for (x, y) in room_one.inner() {
        *dungeon.get_mut(x, y) = Tile::from_type(TileType::Floor);
    }
    for (x, y) in room_two.inner() {
        *dungeon.get_mut(x, y) = Tile::from_type(TileType::Floor);
    }

    dungeon
}

