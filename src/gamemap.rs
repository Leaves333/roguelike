use ratatui::style::Color;

use crate::app::Renderable;

pub struct Tile {
    walkable: bool,
    transparent: bool,
    dark: Renderable,
}

pub enum TileType {
    Floor,
    Wall,
}

impl Tile {
    pub fn new(walkable: bool, transparent: bool, dark: Renderable) -> Self {
        Self {
            walkable,
            transparent,
            dark,
        }
    }

    pub fn from_type(tile_type: TileType) -> Self {
        match tile_type {
            TileType::Wall => Self {
                walkable: false,
                transparent: false,
                dark: Renderable {
                    glyph: '#',
                    fg: Color::White,
                    bg: Color::Reset,
                },
            },
            TileType::Floor => Self {
                walkable: true,
                transparent: true,
                dark: Renderable {
                    glyph: '.',
                    fg: Color::Gray,
                    bg: Color::Reset,
                },
            },
        }
    }
}

pub struct GameMap {
    width: u16,
    height: u16,
}

impl GameMap {}
