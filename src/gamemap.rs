use std::panic;

use crate::{
    app::PLAYER,
    components::{Position, Renderable},
};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

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
    pub level: u16,
    pub object_ids: Vec<usize>,
    pub tiles: Vec<Tile>,
    pub visible: Vec<bool>,
    pub explored: Vec<bool>,
}

impl GameMap {
    pub fn new(width: u16, height: u16, level: u16, object_ids: Vec<usize>) -> Self {
        Self {
            width,
            height,
            level,
            object_ids,
            tiles: vec![Tile::new(TileType::Wall); (width * height) as usize],
            visible: vec![false; (width * height) as usize],
            explored: vec![false; (width * height) as usize],
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

    // quickly check if an index is in bounds
    pub fn in_bounds(&self, x: i16, y: i16) -> bool {
        return 0 <= x && x < self.width as i16 && 0 <= y && y < self.height as i16;
    }

    // finds the position of an blocking object by its id
    pub fn get_blocker_position(&self, id: usize) -> Option<Position> {
        // loop through all tiles to find the object
        // TODO: change this to use a cache
        let mut ids = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.get_ref(x, y);
                if let Some(object_id) = tile.blocker {
                    ids.push(object_id);
                    if object_id == id {
                        return Some(Position { x, y });
                    }
                }
            }
        }
        None
    }

    /// finds the position of an item by its id
    pub fn get_item_position(&self, id: usize) -> Option<Position> {
        // loop through all tiles to find the object
        // TODO: change this to use a cache
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.get_ref(x, y);
                if let Some(object_id) = tile.item {
                    if object_id == id {
                        return Some(Position { x, y });
                    }
                }
            }
        }
        None
    }

    /// gets the position of either a blocker or item matching that id
    pub fn get_position(&self, id: usize) -> Option<Position> {
        self.get_blocker_position(id).or(self.get_item_position(id))
    }

    /// gets the player's position. should be ok to unwrap since player should always exist
    pub fn get_player_position(&self) -> Position {
        self.get_position(PLAYER).unwrap()
    }

    /// attempts to place an object at a specified location.
    /// panics if unsuccessful
    pub fn place_blocker(&mut self, id: usize, x: u16, y: u16) {
        let tile = self.get_mut(x, y);
        if tile.is_walkable() && tile.blocker.is_none() {
            tile.blocker = Some(id);
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
            id
        } else {
            panic!("failed to remove item!")
        }
    }
}
