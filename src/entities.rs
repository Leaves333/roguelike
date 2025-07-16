// this file contains a list of spawnable entities

use crate::components::{Object, Player, Position, Renderable};
use ratatui::style::Color;

pub fn player(x: u16, y: u16) -> (Player, Object) {
    (
        Player {},
        Object {
            name: String::from("Player"),
            position: Position { x, y },
            renderable: Renderable {
                glyph: '@',
                fg: Color::default(),
                bg: Color::Reset,
            },
            blocks_movement: true,
        },
    )
}

pub fn orc(x: u16, y: u16) -> (Object,) {
    (Object {
        name: String::from("Orc"),
        position: Position { x, y },
        renderable: Renderable {
            glyph: 'o',
            fg: Color::Red,
            bg: Color::Reset,
        },
        blocks_movement: true,
    },)
}

pub fn troll(x: u16, y: u16) -> (Object,) {
    (Object {
        name: String::from("Troll"),
        position: Position { x, y },
        renderable: Renderable {
            glyph: 'T',
            fg: Color::Green,
            bg: Color::Reset,
        },
        blocks_movement: true,
    },)
}
