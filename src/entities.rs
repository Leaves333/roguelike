// this file contains a list of spawnable entities

use crate::app::{Player, Position, Renderable};
use ratatui::style::Color;

pub fn player(x: u16, y: u16) -> (Player, Position, Renderable) {
    (
        Player {},
        Position { x, y },
        Renderable {
            glyph: '@',
            fg: Color::default(),
            bg: Color::Reset,
        },
    )
}

pub fn orc(x: u16, y: u16) -> (Position, Renderable) {
    (
        Position { x, y },
        Renderable {
            glyph: 'o',
            fg: Color::Red,
            bg: Color::Reset,
        },
    )
}

pub fn troll(x: u16, y: u16) -> (Position, Renderable) {
    (
        Position { x, y },
        Renderable {
            glyph: 'T',
            fg: Color::Green,
            bg: Color::Reset,
        },
    )
}
