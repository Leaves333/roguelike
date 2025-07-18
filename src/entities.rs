// this file contains a list of spawnable entities

use crate::components::{Fighter, MeleeAI, Object, Player, Position, Renderable};
use ratatui::style::Color;

pub fn player(x: u16, y: u16) -> (Player, Object, Fighter) {
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
        {
            let max_hp = 20;
            let defense = 0;
            let power = 2;
            Fighter::new(max_hp, defense, power)
        },
    )
}

pub fn orc(x: u16, y: u16) -> (Object, Fighter, MeleeAI) {
    (
        Object {
            name: String::from("Orc"),
            position: Position { x, y },
            renderable: Renderable {
                glyph: 'o',
                fg: Color::Red,
                bg: Color::Reset,
            },
            blocks_movement: true,
        },
        {
            let max_hp = 6;
            let defense = 0;
            let power = 2;
            Fighter::new(max_hp, defense, power)
        },
        MeleeAI {},
    )
}

pub fn troll(x: u16, y: u16) -> (Object, Fighter, MeleeAI) {
    (
        Object {
            name: String::from("Troll"),
            position: Position { x, y },
            renderable: Renderable {
                glyph: 'T',
                fg: Color::Green,
                bg: Color::Reset,
            },
            blocks_movement: true,
        },
        {
            let max_hp = 10;
            let defense = 1;
            let power = 4;
            Fighter::new(max_hp, defense, power)
        },
        MeleeAI {},
    )
}
