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
