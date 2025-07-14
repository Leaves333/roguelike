use ratatui::style::Color;

use crate::app::Renderable;

#[derive(Clone)]
pub struct Tile {
    pub walkable: bool,
    pub transparent: bool,
    pub dark: Renderable,
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

    pub fn from_type(tile_type: TileType) -> Tile {
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
    pub width: u16,
    pub height: u16,
    tiles: Vec<Tile>,
}

impl GameMap {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::from_type(TileType::Floor); (width * height) as usize],
        }
    }

    // get a reference to a tile of the gamemap
    pub fn get_ref(&self, x: u16, y: u16) -> &Tile {
        return &self.tiles[(x + y * self.width) as usize];
    }

    // get a mutable reference to a tile of the gamemap
    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Tile {
        return &mut self.tiles[(x + y * self.width) as usize];
    }

    pub fn in_bounds(&self, x: u16, y: u16) -> bool {
        return x < self.width && y < self.height;
    }
}
