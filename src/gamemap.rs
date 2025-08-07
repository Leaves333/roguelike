use crate::components::Renderable;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Tile {
    pub walkable: bool,
    pub transparent: bool,
    pub light: Renderable,
    pub dark: Renderable,
}

pub enum TileType {
    Floor,
    Wall,
}

// the default renderable to display for a tile when it is not explored and not visible
pub fn shroud_renderable() -> Renderable {
    Renderable {
        glyph: ' ',
        fg: Color::Reset,
        bg: Color::Reset,
    }
}

impl Tile {
    /// constructs a tile from a TileType enum
    pub fn from_type(tile_type: TileType) -> Tile {
        match tile_type {
            TileType::Wall => Self {
                walkable: false,
                transparent: false,
                light: Renderable {
                    glyph: '#',
                    fg: Color::Gray,
                    bg: Color::Reset,
                },
                dark: Renderable {
                    glyph: '#',
                    fg: Color::DarkGray,
                    bg: Color::Reset,
                },
            },
            TileType::Floor => Self {
                walkable: true,
                transparent: true,
                light: Renderable {
                    glyph: '.',
                    fg: Color::Gray,
                    bg: Color::Reset,
                },
                dark: Renderable {
                    glyph: '.',
                    fg: Color::DarkGray,
                    bg: Color::Reset,
                },
            },
        }
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

#[derive(Serialize, Deserialize)]
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
            tiles: vec![Tile::from_type(TileType::Wall); (width * height) as usize],
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
}
