use ratatui::style::Color;

#[derive(Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
    pub bg: Color,
}

#[derive(Clone)]
pub struct Object {
    pub name: String,
    pub position: Position,
    pub renderable: Renderable,
    pub blocks_movement: bool,
}

pub struct Player {}

#[derive(Clone)]
pub struct Fighter {
    pub max_hp: u16,
    hp: u16,
    pub defense: i16,
    pub power: i16,
}

impl Fighter {
    pub fn new(max_hp: u16, defense: i16, power: i16) -> Self {
        Self {
            max_hp,
            hp: max_hp,
            defense,
            power,
        }
    }

    pub fn get_hp(&self) -> u16 {
        self.hp
    }

    pub fn set_hp(&mut self, value: u16) {
        self.hp = value.min(self.max_hp);
    }
}

pub struct MeleeAI {
    // pub awake: bool,
    // pub target_pos: Position,
}
