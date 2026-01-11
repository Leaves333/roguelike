// this file contains a list of spawnable entities

use crate::components::{
    AIType, DeathCallback, Equipment, Fighter, Item, MeleeAIData, Object, RenderLayer, Renderable,
    Slot,
};
use ratatui::style::Color;

pub fn stairs() -> Object {
    let name = "Stairs".to_string();
    let tooltip = "stairs leading to the next floor".to_string();

    let renderable = Renderable {
        glyph: '>',
        fg: Color::Gray,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer)
}

pub fn player() -> Object {
    let name = "Player".to_string();
    let tooltip = "this is you :D".to_string();

    let renderable = Renderable {
        glyph: '@',
        fg: Color::default(),
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Blocking;

    Object::new(name, tooltip, renderable, render_layer).set_fighter({
        let max_hp = 20;
        let defense = 0;
        let power = 2;
        Fighter::new(max_hp, defense, power, DeathCallback::Player)
    })
}

pub fn orc() -> Object {
    let name = "Orc".to_string();
    let tooltip = "orcs are evil creatures :(".to_string();

    let renderable = Renderable {
        glyph: 'o',
        fg: Color::Red,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Blocking;
    let ai_component = AIType::Melee(MeleeAIData::new());

    Object::new(name, tooltip, renderable, render_layer)
        .set_fighter({
            let max_hp = 6;
            let defense = 0;
            let power = 2;
            Fighter::new(max_hp, defense, power, DeathCallback::Monster)
        })
        .set_ai(ai_component)
}

pub fn rat() -> Object {
    let name = "Rat".to_string();
    let tooltip = "speedy evil creature".to_string();

    let renderable = Renderable {
        glyph: 'r',
        fg: Color::Yellow,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Blocking;
    let ai_component = AIType::Melee(MeleeAIData::new().set_move_speed(75).set_attack_speed(75));

    Object::new(name, tooltip, renderable, render_layer)
        .set_fighter({
            let max_hp = 5;
            let defense = 0;
            let power = 2;
            Fighter::new(max_hp, defense, power, DeathCallback::Monster)
        })
        .set_ai(ai_component)
}

pub fn troll() -> Object {
    let name = "Troll".to_string();
    let tooltip = "slow and heavy creature".to_string();

    let renderable = Renderable {
        glyph: 'T',
        fg: Color::Green,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Blocking;
    let ai_component = AIType::Melee(MeleeAIData::new().set_move_speed(150).set_attack_speed(150));

    Object::new(name, tooltip, renderable, render_layer)
        .set_fighter({
            let max_hp = 10;
            let defense = 1;
            let power = 5;
            Fighter::new(max_hp, defense, power, DeathCallback::Monster)
        })
        .set_ai(ai_component)
}

pub fn weapon_dagger() -> Object {
    let name = "dagger".to_string();
    let tooltip = "a small dagger".to_string();

    let renderable = Renderable {
        glyph: '(',
        fg: Color::default(),
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer)
        .set_item(Item::Equipment)
        .set_equipment(Equipment {
            slot: Slot::Weapon,
            power_bonus: 2,
            defense_bonus: 0,
        })
}

pub fn weapon_longsword() -> Object {
    let name = "longsword".to_string();
    let tooltip = "a large longsword".to_string();

    let renderable = Renderable {
        glyph: '(',
        fg: Color::Blue,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer)
        .set_item(Item::Equipment)
        .set_equipment(Equipment {
            slot: Slot::Weapon,
            power_bonus: 4,
            defense_bonus: 0,
        })
}

pub fn helmet() -> Object {
    let name = String::from("helmet");
    let tooltip = "a sturdy helmet".to_string();

    let renderable = Renderable {
        glyph: ']',
        fg: Color::default(),
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer)
        .set_item(Item::Equipment)
        .set_equipment(Equipment {
            slot: Slot::Head,
            power_bonus: 0,
            defense_bonus: 1,
        })
}

pub fn leather_armor() -> Object {
    let name = "leather armor".to_string();
    let tooltip = "supple leather armor".to_string();

    let renderable = Renderable {
        glyph: '[',
        fg: Color::default(),
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer)
        .set_item(Item::Equipment)
        .set_equipment(Equipment {
            slot: Slot::Body,
            power_bonus: 0,
            defense_bonus: 1,
        })
}

pub fn plate_armor() -> Object {
    let name = "plate armor".to_string();
    let tooltip = "sturdy plate armor".to_string();

    let renderable = Renderable {
        glyph: '[',
        fg: Color::Blue,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer)
        .set_item(Item::Equipment)
        .set_equipment(Equipment {
            slot: Slot::Body,
            power_bonus: 0,
            defense_bonus: 2,
        })
}
