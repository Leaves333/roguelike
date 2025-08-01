// this file contains a list of spawnable entities

use crate::components::{AIType, DeathCallback, Fighter, Item, Object, RenderStatus, Renderable};
use ratatui::style::Color;

pub fn spawn(x: u16, y: u16, mut object: Object) -> Object {
    object.pos.x = x;
    object.pos.y = y;
    object
}

pub fn player() -> Object {
    let name = String::from("Player");
    let renderable = Renderable {
        glyph: '@',
        fg: Color::default(),
        bg: Color::Reset,
    };
    let render_status = RenderStatus::AlwaysShow;
    let blocks_movement = true;
    let alive = true;

    Object::new(name, renderable, render_status, blocks_movement, alive).set_fighter({
        let max_hp = 20;
        let defense = 0;
        let power = 2;
        Fighter::new(max_hp, defense, power, DeathCallback::Player)
    })
}

pub fn orc() -> Object {
    let name = String::from("Orc");
    let renderable = Renderable {
        glyph: 'o',
        fg: Color::Red,
        bg: Color::Reset,
    };
    let render_status = RenderStatus::ShowInFOV;
    let blocks_movement = true;
    let alive = true;

    Object::new(name, renderable, render_status, blocks_movement, alive)
        .set_fighter({
            let max_hp = 5;
            let defense = 0;
            let power = 2;
            Fighter::new(max_hp, defense, power, DeathCallback::Monster)
        })
        .set_ai(AIType::Melee)
}

pub fn troll() -> Object {
    let name = String::from("Troll");
    let renderable = Renderable {
        glyph: 'T',
        fg: Color::Green,
        bg: Color::Reset,
    };
    let render_status = RenderStatus::ShowInFOV;
    let blocks_movement = true;
    let alive = true;

    Object::new(name, renderable, render_status, blocks_movement, alive)
        .set_fighter({
            let max_hp = 8;
            let defense = 1;
            let power = 4;
            Fighter::new(max_hp, defense, power, DeathCallback::Monster)
        })
        .set_ai(AIType::Melee)
}

pub fn potion_cure_wounds() -> Object {
    let name = String::from("potion of cure wounds");
    let renderable = Renderable {
        glyph: '!',
        fg: Color::Magenta,
        bg: Color::Reset,
    };
    let render_status = RenderStatus::ShowInFOV;
    let blocks_movement = false;
    let alive = false;

    Object::new(name, renderable, render_status, blocks_movement, alive).set_item(Item::Heal)
}

pub fn scroll_lightning() -> Object {
    let name = String::from("scroll of lightning");
    let renderable = Renderable {
        glyph: '?',
        fg: Color::Blue,
        bg: Color::Reset,
    };
    let render_status = RenderStatus::ShowInFOV;
    let blocks_movement = false;
    let alive = false;

    Object::new(name, renderable, render_status, blocks_movement, alive).set_item(Item::Lightning)
}
