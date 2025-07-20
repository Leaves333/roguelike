use ratatui::style::Color;

#[derive(Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn default() -> Self {
        Position { x: 0, y: 0 }
    }
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
    pub pos: Position,
    pub renderable: Renderable,
    pub blocks_movement: bool,
    pub alive: bool,
    pub fighter: Option<Fighter>,
    pub ai: Option<AIType>,
}

#[derive(Clone)]
pub struct Fighter {
    pub max_hp: u16,
    pub hp: u16,
    pub defense: i16,
    pub power: i16,
    pub death_callback: DeathCallback,
}

impl Fighter {
    pub fn new(max_hp: u16, defense: i16, power: i16, death_callback: DeathCallback) -> Self {
        Self {
            max_hp,
            hp: max_hp,
            defense,
            power,
            death_callback,
        }
    }
}

#[derive(Clone)]
pub enum AIType {
    Melee,
}

#[derive(Clone)]
pub enum DeathCallback {
    Player,
    Monster,
}
