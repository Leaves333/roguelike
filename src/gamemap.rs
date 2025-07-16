use std::collections::HashSet;

use hecs::{Entity, World};
use ratatui::style::Color;

use crate::{
    components::{Object, Renderable},
    los,
};

#[derive(Clone)]
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
    pub fn new(walkable: bool, transparent: bool, light: Renderable, dark: Renderable) -> Self {
        Self {
            walkable,
            transparent,
            light,
            dark,
        }
    }

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

pub struct GameMap {
    pub width: u16,
    pub height: u16,
    pub world: World,
    tiles: Vec<Tile>,
    visible: Vec<bool>,
    explored: Vec<bool>,
}

impl GameMap {
    pub fn new(width: u16, height: u16, world: World) -> Self {
        Self {
            width,
            height,
            world,
            tiles: vec![Tile::from_type(TileType::Wall); (width * height) as usize],
            visible: vec![false; (width * height) as usize],
            explored: vec![false; (width * height) as usize],
        }
    }

    // get a reference to a tile of the gamemap
    pub fn get_ref(&self, x: u16, y: u16) -> &Tile {
        return &self.tiles[self.idx(x, y)];
    }

    // get a mutable reference to a tile of the gamemap
    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Tile {
        let idx = self.idx(x, y);
        return &mut self.tiles[idx];
    }

    pub fn is_visible(&self, x: u16, y: u16) -> bool {
        self.visible[self.idx(x, y)]
    }

    pub fn set_visible(&mut self, x: u16, y: u16, value: bool) {
        let idx = self.idx(x, y);
        self.visible[idx] = value;
    }

    pub fn is_explored(&self, x: u16, y: u16) -> bool {
        self.explored[self.idx(x, y)]
    }

    pub fn set_explored(&mut self, x: u16, y: u16, value: bool) {
        let idx = self.idx(x, y);
        self.explored[idx] = value;
    }

    // quickly check if an index is in bounds
    pub fn in_bounds(&self, x: i16, y: i16) -> bool {
        return 0 <= x && x < self.width as i16 && 0 <= y && y < self.height as i16;
    }

    pub fn get_blocking_entity_at_location(&self, x: u16, y: u16) -> Option<Entity> {
        for (entity, obj) in self.world.query::<&Object>().iter() {
            if obj.blocks_movement && obj.position.x == x && obj.position.y == y {
                return Some(entity);
            }
        }
        return None;
    }

    // recompute visible area based on the player's fov
    pub fn update_fov(&mut self, player: Entity, radius: u16) {
        // TODO: use a different symmetric algo to calculate line of sight

        let player_object = self.world.get::<&Object>(player).unwrap();
        let position = &player_object.position;
        let (player_x, player_y) = (position.x, position.y);
        drop(player_object);

        self.visible.fill(false);

        // calculate bounds for visibility
        let (xlow, xhigh) = (
            (player_x.saturating_sub(radius)).max(0),
            (player_x + radius).min(self.width - 1),
        );
        let (ylow, yhigh) = (
            (player_y.saturating_sub(radius)).max(0),
            (player_y + radius).min(self.width - 1),
        );

        // loop through each x, y to check visibility
        let mut visited = HashSet::new();
        for target_x in xlow..=xhigh {
            for target_y in ylow..=yhigh {
                // already checked this square
                if visited.contains(&(target_x, target_y)) {
                    continue;
                }

                // calculate los path from player to target square
                let path: Vec<(u16, u16)> = los::bresenham(
                    (player_x.into(), player_y.into()),
                    (target_x.into(), target_y.into()),
                )
                .iter()
                .map(|&(x, y)| (x as u16, y as u16))
                .collect();

                // walk along the path to check for visibility
                for (x, y) in path {
                    visited.insert((x, y));
                    if !self.get_ref(x, y).transparent {
                        self.set_visible(x, y, true);
                        break;
                    }
                    self.set_visible(x, y, true);
                }
            }
        }

        // explored |= visible
        for (e, &v) in self.explored.iter_mut().zip(self.visible.iter()) {
            *e |= v;
        }
    }

    // helper private function for indexing the arrays
    fn idx(&self, x: u16, y: u16) -> usize {
        (x + y * self.width) as usize
    }
}
