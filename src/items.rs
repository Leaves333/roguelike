use ratatui::style::Color;

use crate::{
    app::{App, PLAYER},
    components::{Item, Object, Position, RenderLayer, Renderable},
    engine::{self, UseResult, damage, defense, heal, take_damage},
};

/// this file contains consumable items and their associated effects when used

const HEAL_AMOUNT: u16 = 10;
pub fn potion_cure_wounds() -> Object {
    let name = "potion of cure wounds".to_string();
    let tooltip = format!("heals the player for {HEAL_AMOUNT} base health.");

    let renderable = Renderable {
        glyph: '!',
        fg: Color::Magenta,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer).set_item(Item::Heal)
}

/// effects of a potion of healing. heals the player
pub fn cast_cure_wounds(app: &mut App) -> UseResult {
    let fighter = match &app.objects.get(&PLAYER).unwrap().fighter {
        Some(x) => x,
        None => {
            panic!("trying to cast heal, but target_id does not have a fighter component!")
        }
    };

    if fighter.hp == fighter.max_hp {
        app.add_to_log(
            String::from("You are already at full health."),
            Color::default(),
        );
        UseResult::Cancelled
    } else {
        heal(app, PLAYER, HEAL_AMOUNT);
        app.add_to_log(
            String::from("Your wounds start to close."),
            Color::default(),
        );
        UseResult::UsedUp
    }
}

const LIGHTNING_DAMAGE: i16 = 8;
/// scroll of lightning smites a chosen target within line of sight
pub fn scroll_lightning() -> Object {
    let name = "scroll of lightning".to_string();
    let tooltip =
        format!("smites an enemy with lightning, dealing {LIGHTNING_DAMAGE} base damage.");

    let renderable = Renderable {
        glyph: '?',
        fg: Color::Cyan,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer).set_item(Item::Lightning)
}

pub fn cast_lightning(app: &mut App, target: Position) -> UseResult {
    let target_id = match engine::get_smite_target(app, target) {
        Some(x) => {
            if x == PLAYER {
                app.add_to_log(String::from("Can't target yourself!"), Color::default());
                return UseResult::Cancelled;
            } else {
                x
            }
        }
        None => {
            app.add_to_log(String::from("No targets there."), Color::default());
            return UseResult::Cancelled;
        }
    };

    let _fighter = match &app.objects.get(&target_id).unwrap().fighter {
        Some(x) => x,
        None => {
            panic!("trying to cast lightning, but target_id does not have a fighter component!")
        }
    };

    let damage_dealt = damage(LIGHTNING_DAMAGE, defense(app, target_id));

    let target_obj = app.objects.get(&target_id).unwrap();
    let attack_desc = format!("Lightning smites the {}", target_obj.name);

    if damage_dealt > 0 {
        app.add_to_log(
            format!("{} for {} damage.", attack_desc, damage_dealt),
            Color::LightBlue,
        );
        take_damage(app, target_id, damage_dealt as u16);
    } else {
        app.add_to_log(
            format!("{} but does no damage.", attack_desc),
            Color::default(),
        );
    }

    UseResult::UsedUp
}

const HEXBOLT_DAMAGE: i16 = 5;
/// scroll of hexbolt fires a projectile in a line
pub fn scroll_hexbolt() -> Object {
    let name = "scroll of hexbolt".to_string();
    let tooltip = format!(
        "fires a hexbolt at an enemy, colliding with the first object and dealing {HEXBOLT_DAMAGE} base damage"
    );

    let renderable = Renderable {
        glyph: '?',
        fg: Color::Blue,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer).set_item(Item::Hexbolt)
}

pub fn cast_hexbolt(app: &mut App, target: Position) -> UseResult {
    let player_pos = app.gamemap.get_position(PLAYER).unwrap();
    if target == player_pos {
        app.add_to_log(String::from("Can't target yourself!"), Color::default());
        return UseResult::Cancelled;
    }

    let targets = engine::get_line_target(app, target);
    let target_id = match targets.iter().nth(0) {
        Some(x) => x.clone(),
        None => {
            app.add_to_log(String::from("No enemies targeted."), Color::default());
            return UseResult::Cancelled;
        }
    };

    if app.objects.get(&target_id).unwrap().fighter.is_none() {
        panic!("trying to cast hexbolt, but target_id does not have a fighter component!")
    }

    let damage_dealt = damage(HEXBOLT_DAMAGE, defense(app, target_id));

    let target_obj = app.objects.get(&target_id).unwrap();
    let attack_desc = format!("The hexbolt blasts the {}", target_obj.name);

    if damage_dealt > 0 {
        app.add_to_log(
            format!("{} for {} damage.", attack_desc, damage_dealt),
            Color::LightBlue,
        );
        take_damage(app, target_id, damage_dealt as u16);
    } else {
        app.add_to_log(
            format!("{} but does no damage.", attack_desc),
            Color::default(),
        );
    }

    UseResult::UsedUp
}

pub fn scroll_fireball() -> Object {
    let name = "scroll of fireball".to_string();
    let tooltip = "lauches a massive fireball at an enemy, causing an explosion".to_string();

    let renderable = Renderable {
        glyph: '?',
        fg: Color::Red,
        bg: Color::Reset,
    };
    let render_layer = RenderLayer::Item;

    Object::new(name, tooltip, renderable, render_layer).set_item(Item::Fireball)
}
