// this file contains a list of spawnable entities

use crate::components::{
    AIType, DeathCallback, Equipment, Fighter, Item, Object, RenderLayer, RenderStatus, Renderable,
    Slot,
};
use ratatui::style::Color;

pub fn spawn(x: u16, y: u16, mut object: Object) -> Object {
    object.pos.x = x;
    object.pos.y = y;
    object
}

pub fn stairs() -> Object {
    let name = String::from("Stairs");
    let renderable = Renderable {
        glyph: '>',
        fg: Color::Gray,
        bg: Color::Reset,
    };
    let render_status = RenderStatus::ShowInExplored;
    let render_layer = RenderLayer::Item;
    let blocks_movement = false;
    let alive = false;

    Object::new(
        name,
        renderable,
        render_status,
        render_layer,
        blocks_movement,
        alive,
    )
}

pub fn player() -> Object {
    let name = String::from("Player");
    let renderable = Renderable {
        glyph: '@',
        fg: Color::default(),
        bg: Color::Reset,
    };
    let render_status = RenderStatus::ShowInExplored;
    let render_layer = RenderLayer::Blocking;
    let blocks_movement = true;
    let alive = true;

    Object::new(
        name,
        renderable,
        render_status,
        render_layer,
        blocks_movement,
        alive,
    )
    .set_fighter({
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
    let render_layer = RenderLayer::Blocking;
    let blocks_movement = true;
    let alive = true;

    Object::new(
        name,
        renderable,
        render_status,
        render_layer,
        blocks_movement,
        alive,
    )
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
    let render_layer = RenderLayer::Blocking;
    let blocks_movement = true;
    let alive = true;

    Object::new(
        name,
        renderable,
        render_status,
        render_layer,
        blocks_movement,
        alive,
    )
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
    let render_layer = RenderLayer::Item;
    let blocks_movement = false;
    let alive = false;

    Object::new(
        name,
        renderable,
        render_status,
        render_layer,
        blocks_movement,
        alive,
    )
    .set_item(Item::Heal)
}

pub fn scroll_lightning() -> Object {
    let name = String::from("scroll of lightning");
    let renderable = Renderable {
        glyph: '?',
        fg: Color::Blue,
        bg: Color::Reset,
    };
    let render_status = RenderStatus::ShowInFOV;
    let render_layer = RenderLayer::Item;
    let blocks_movement = false;
    let alive = false;

    Object::new(
        name,
        renderable,
        render_status,
        render_layer,
        blocks_movement,
        alive,
    )
    .set_item(Item::Lightning)
}

pub fn weapon_dagger() -> Object {
    let name = String::from("dagger");
    let renderable = Renderable {
        glyph: '(',
        fg: Color::default(),
        bg: Color::Reset,
    };
    let render_status = RenderStatus::ShowInFOV;
    let render_layer = RenderLayer::Item;
    let blocks_movement = false;
    let alive = false;

    Object::new(
        name,
        renderable,
        render_status,
        render_layer,
        blocks_movement,
        alive,
    )
    .set_item(Item::Equipment)
    .set_equipment(Equipment {
        slot: Slot::Weapon,
        power_bonus: 2,
        defense_bonus: 0,
    })
}
