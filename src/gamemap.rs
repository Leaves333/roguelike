use std::{
    collections::{HashMap, HashSet, VecDeque},
    panic,
};

use crate::components::{Position, Renderable};

use rand::{rng, seq::SliceRandom};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

const ITEM_DROP_RADIUS: u16 = 2;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Tile {
    pub tile_type: TileType,
    pub item: Option<usize>,
    pub blocker: Option<usize>,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            item: None,
            blocker: None,
        }
    }

    // NOTE: walkable tiles are those on which items and blockers can be placed
    pub fn is_walkable(&self) -> bool {
        match self.tile_type {
            TileType::Floor => true,
            TileType::Wall => false,
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self.tile_type {
            TileType::Floor => true,
            TileType::Wall => false,
        }
    }

    pub fn renderable(&self) -> Renderable {
        match self.tile_type {
            TileType::Wall => Renderable {
                glyph: '#',
                fg: Color::Gray,
                bg: Color::Reset,
            },
            TileType::Floor => Renderable {
                glyph: '.',
                fg: Color::Gray,
                bg: Color::Reset,
            },
        }
    }
}

// the default renderable to display for a tile when it is not explored and not visible
pub fn shroud_renderable() -> Renderable {
    Renderable {
        glyph: ' ',
        fg: Color::Reset,
        bg: Color::Reset,
    }
}

// generic functions to use
pub fn coords_to_idx(x: u16, y: u16, width: u16) -> usize {
    (x + y * width) as usize
}

pub fn idx_to_coords(idx: usize, width: u16) -> (u16, u16) {
    let idx = idx as u16;
    (idx % width, idx / width)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameMap {
    pub width: u16,
    pub height: u16,
    pub level: u16, // the "depth" of the dungeon floor, determining its difficulty
    pub tiles: Vec<Tile>, // the tiles comprising the map of the dungeon
    pub visible: Vec<bool>, // whether any given tile is visible
    pub explored: Vec<bool>, // whether any given tile has been explored
    pub last_seen: Vec<Renderable>, // the state of the tile when it was last seen
    objects: HashMap<usize, Position>, // objects present in this gamemap, mapped to their position
}

impl GameMap {
    pub fn new(width: u16, height: u16, level: u16) -> Self {
        Self {
            width,
            height,
            level,
            tiles: vec![Tile::new(TileType::Wall); (width * height) as usize],
            visible: vec![false; (width * height) as usize],
            explored: vec![false; (width * height) as usize],
            last_seen: vec![Renderable::default(); (width * height) as usize],
            objects: HashMap::new(),
        }
    }

    // get a reference to a tile of the gamemap
    pub fn get_ref(&self, x: u16, y: u16) -> &Tile {
        return &self.tiles[coords_to_idx(x, y, self.width)];
    }

    // get a mutable reference to a tile of the gamemap
    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Tile {
        return &mut self.tiles[coords_to_idx(x, y, self.width)];
    }

    pub fn is_visible(&self, x: u16, y: u16) -> bool {
        self.visible[coords_to_idx(x, y, self.width)]
    }

    pub fn set_visible(&mut self, x: u16, y: u16, value: bool) {
        self.visible[coords_to_idx(x, y, self.width)] = value;
    }

    pub fn is_explored(&self, x: u16, y: u16) -> bool {
        self.explored[coords_to_idx(x, y, self.width)]
    }

    #[allow(dead_code)]
    pub fn set_explored(&mut self, x: u16, y: u16, value: bool) {
        self.explored[coords_to_idx(x, y, self.width)] = value;
    }

    /// returns a copy of the last seen version of a given tile
    pub fn get_last_seen(&self, x: u16, y: u16) -> Renderable {
        self.last_seen[coords_to_idx(x, y, self.width)].clone()
    }

    pub fn set_last_seen(&mut self, x: u16, y: u16, value: Renderable) {
        self.last_seen[coords_to_idx(x, y, self.width)] = value;
    }

    // quickly check if an index is in bounds
    pub fn in_bounds(&self, x: i16, y: i16) -> bool {
        return 0 <= x && x < self.width as i16 && 0 <= y && y < self.height as i16;
    }

    /// gets the position of either a blocker or item matching that id
    pub fn get_position(&self, id: usize) -> Option<Position> {
        self.objects.get(&id).copied()
    }

    /// attempts to place an object at a specified location.
    /// panics if unsuccessful
    pub fn place_blocker(&mut self, id: usize, x: u16, y: u16) {
        let tile = self.get_mut(x, y);
        if tile.is_walkable() && tile.blocker.is_none() {
            tile.blocker = Some(id);
            self.objects.insert(id, Position { x, y });
        } else {
            panic!("failed to place blocker!")
        }
    }

    /// attempts to place an item at a specified location.
    /// panics if unsuccessful
    pub fn place_item(&mut self, id: usize, x: u16, y: u16) {
        let tile = self.get_mut(x, y);
        if tile.is_walkable() && tile.item.is_none() {
            tile.item = Some(id);
            self.objects.insert(id, Position { x, y });
        } else {
            panic!("failed to place item!")
        }
    }

    /// removes a blocker from a specified location
    /// returns the id of the removed blocker if there was one
    /// panics if there was no blocker there
    pub fn remove_blocker(&mut self, x: u16, y: u16) -> usize {
        let tile = self.get_mut(x, y);
        if let Some(id) = tile.blocker {
            tile.blocker = None;
            self.objects.remove(&id);
            id
        } else {
            panic!("failed to remove blocker!")
        }
    }

    /// removes a item from a specified location
    /// returns the id of the removed item if there was one
    /// panics if there was no item there
    pub fn remove_item(&mut self, x: u16, y: u16) -> usize {
        let tile = self.get_mut(x, y);
        if let Some(id) = tile.item {
            tile.item = None;
            self.objects.remove(&id);
            id
        } else {
            panic!("failed to remove item!")
        }
    }

    /// attempts to place an item at a given location, or somewhere nearby if possible
    /// returns the position that the item was added to
    pub fn area_place_item(&mut self, x: u16, y: u16, id: usize) -> Option<Position> {
        let mut visited: HashSet<(u16, u16)> = HashSet::new();
        let mut queue: VecDeque<(u16, u16)> = VecDeque::new();
        queue.push_back((x, y));
        visited.insert((x, y));

        while !queue.is_empty() {
            let (cur_x, cur_y) = queue.pop_front().unwrap();

            // if its out of our target drop range, don't drop it
            let dist = cur_x.abs_diff(x).max(cur_y.abs_diff(y));
            if dist > ITEM_DROP_RADIUS {
                continue;
            }

            // if the tile is not walkable, don't drop it
            if !self.get_ref(cur_x, cur_y).is_walkable() {
                continue;
            }

            // if there is space to drop it, drop at this location
            let tile = self.get_mut(cur_x, cur_y);
            if tile.item.is_none() {
                self.place_item(id, cur_x, cur_y);
                return Some(Position { x: cur_x, y: cur_y });
            }

            // directions are shuffled to add some randomness to how items drop
            let mut dirs = [(-1, 0), (1, 0), (0, -1), (0, 1)];
            let mut rng = rng();
            dirs.shuffle(&mut rng);

            for (dx, dy) in dirs {
                let (new_x, new_y) = (cur_x as i16 + dx, cur_y as i16 + dy);
                if !self.in_bounds(new_x, new_y) {
                    continue;
                }

                let (new_x, new_y) = (new_x as u16, new_y as u16);
                if visited.contains(&(new_x, new_y)) {
                    continue;
                }

                visited.insert((new_x, new_y));
                queue.push_back((new_x, new_y));
            }
        }

        return None;
    }
}
