// this file contains a list of spawnable entities

use crate::components::{AIType, Fighter, Object, Position, Renderable};
use ratatui::style::Color;

pub fn spawn(x: u16, y: u16, mut object: Object) -> Object {
    object.position.x = x;
    object.position.y = y;
    object
}

pub fn player() -> Object {
    Object {
        name: String::from("Player"),
        position: Position::default(),
        renderable: Renderable {
            glyph: '@',
            fg: Color::default(),
            bg: Color::Reset,
        },
        blocks_movement: true,
        alive: true,
        fighter: Some({
            let max_hp = 20;
            let defense = 0;
            let power = 2;
            Fighter::new(max_hp, defense, power)
        }),
        ai: None,
    }
}

pub fn orc() -> Object {
    Object {
        name: String::from("Orc"),
        position: Position::default(),
        renderable: Renderable {
            glyph: 'o',
            fg: Color::Red,
            bg: Color::Reset,
        },
        blocks_movement: true,
        alive: true,
        fighter: Some({
            let max_hp = 5;
            let defense = 0;
            let power = 2;
            Fighter::new(max_hp, defense, power)
        }),
        ai: Some(AIType::Melee),
    }
}

pub fn troll() -> Object {
    Object {
        name: String::from("Troll"),
        position: Position::default(),
        renderable: Renderable {
            glyph: '@',
            fg: Color::Green,
            bg: Color::Reset,
        },
        blocks_movement: true,
        alive: true,
        fighter: Some({
            let max_hp = 8;
            let defense = 1;
            let power = 4;
            Fighter::new(max_hp, defense, power)
        }),
        ai: Some(AIType::Melee),
    }
}
