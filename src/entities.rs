// this file contains a list of spawnable entities

use crate::components::{
    AIType, DeathCallback, Fighter, Object, Position, RenderStatus, Renderable,
};
use ratatui::style::Color;

pub fn spawn(x: u16, y: u16, mut object: Object) -> Object {
    object.pos.x = x;
    object.pos.y = y;
    object
}

pub fn player() -> Object {
    Object {
        name: String::from("Player"),
        pos: Position::default(),
        renderable: Renderable {
            glyph: '@',
            fg: Color::default(),
            bg: Color::Reset,
        },
        render_status: RenderStatus::AlwaysShow,
        blocks_movement: true,
        alive: true,
        fighter: Some({
            let max_hp = 20;
            let defense = 0;
            let power = 2;
            Fighter::new(max_hp, defense, power, DeathCallback::Player)
        }),
        ai: None,
    }
}

pub fn orc() -> Object {
    Object {
        name: String::from("Orc"),
        pos: Position::default(),
        renderable: Renderable {
            glyph: 'o',
            fg: Color::Red,
            bg: Color::Reset,
        },
        render_status: RenderStatus::ShowInFOV,
        blocks_movement: true,
        alive: true,
        fighter: Some({
            let max_hp = 5;
            let defense = 0;
            let power = 2;
            Fighter::new(max_hp, defense, power, DeathCallback::Monster)
        }),
        ai: Some(AIType::Melee),
    }
}

pub fn troll() -> Object {
    Object {
        name: String::from("Troll"),
        pos: Position::default(),
        renderable: Renderable {
            glyph: 'T',
            fg: Color::Green,
            bg: Color::Reset,
        },
        render_status: RenderStatus::ShowInFOV,
        blocks_movement: true,
        alive: true,
        fighter: Some({
            let max_hp = 8;
            let defense = 1;
            let power = 4;
            Fighter::new(max_hp, defense, power, DeathCallback::Monster)
        }),
        ai: Some(AIType::Melee),
    }
}
