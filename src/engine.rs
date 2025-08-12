use ratatui::style::{Color, Style, Stylize};

use crate::{
    app::{App, GameScreen, Log, ObjectMap, PLAYER},
    components::{DeathCallback, Item, Position, RenderLayer},
    gamemap::GameMap,
};

// NOTE: this crate contains functions that control the gameplay

/// used to determine if an item was sucessfully used
pub enum UseResult {
    UsedUp,
    Equipped,
    Cancelled,
}

/// different targeting modes for targeted abilities
pub enum TargetingMode {
    SmiteEnemy, // smite target any enemy in line of sight
}

pub fn get_blocking_object_id(
    objects: &ObjectMap,
    gamemap: &GameMap,
    pos: Position,
) -> Option<usize> {
    for id in gamemap.object_ids.iter() {
        let obj = &objects.get(id).unwrap();
        if obj.blocks_movement && obj.pos.x == pos.x && obj.pos.y == pos.y {
            return Some(id.clone());
        }
    }
    return None;
}

/// heals an entity for the specified amount
pub fn heal(objects: &mut ObjectMap, id: usize, heal_amount: u16) {
    let obj = objects.get_mut(&id).unwrap();
    if let Some(fighter) = obj.fighter.as_mut() {
        fighter.hp += heal_amount;
        fighter.hp = fighter.hp.min(fighter.max_hp)
    }
}

/// applies damage to an entity for the specified amount
pub fn take_damage(objects: &mut ObjectMap, log: &mut Log, id: usize, damage: u16) {
    let obj = &mut objects.get_mut(&id).unwrap();
    let mut death_callback = None;
    if let Some(fighter) = obj.fighter.as_mut() {
        if damage > 0 {
            fighter.hp = fighter.hp.saturating_sub(damage);
        }

        if fighter.hp <= 0 {
            obj.alive = false;
            death_callback = Some(fighter.death_callback.clone());
        }

        fighter.hp = fighter.hp.min(fighter.max_hp);
    }

    if let Some(callback) = death_callback {
        match callback {
            DeathCallback::Player => player_death(objects, log),
            DeathCallback::Monster => monster_death(objects, log, id),
        }
    }
}

pub fn player_death(objects: &mut ObjectMap, log: &mut Log) {
    let player = &mut objects.get_mut(&PLAYER).unwrap();
    log.add(String::from("You died!"), Style::new().italic().red());

    let renderable = &mut player.renderable;
    renderable.glyph = '%';
    renderable.fg = Color::Red;
}

pub fn monster_death(objects: &mut ObjectMap, log: &mut Log, id: usize) {
    let monster = &mut objects.get_mut(&id).unwrap();
    log.add(format!("{} dies!", monster.name), Color::Red);

    let renderable = &mut monster.renderable;
    renderable.glyph = '%';
    renderable.fg = Color::Red;

    monster.blocks_movement = false;
    monster.render_layer = RenderLayer::Corpse;
    monster.alive = false;
    monster.fighter = None;
    monster.name = format!("remains of {}", monster.name);
}

impl Item {
    pub fn needs_targeting(&self) -> bool {
        match self {
            Item::Lightning => true,
            _ => false,
        }
    }
    /// switches the game screen to the appropriate targeting mode for the item
    pub fn on_targeting(&self, app: &mut App, inventory_idx: usize) {
        // NOTE: need to check if item is targetable before calling this function!
        if !self.needs_targeting() {
            unreachable!()
        }

        let targeting_text = match self {
            Item::Lightning => String::from("Aim the bolt of lightning at what?"),
            _ => {
                unreachable!()
            }
        };

        let targeting_mode = match self {
            Item::Lightning => TargetingMode::SmiteEnemy,
            _ => {
                unreachable!()
            }
        };

        // all other cases, targeting is required
        let targeting = GameScreen::Targeting {
            cursor: app.objects.get(&PLAYER).unwrap().pos,
            targeting: targeting_mode,
            text: targeting_text,
            inventory_idx,
        };

        app.game_screen = targeting;
    }

    /// callback to be used when the item is consumed
    pub fn on_use(&self, app: &mut App, target: Option<Position>) -> UseResult {
        if self.needs_targeting() && target.is_none() {
            panic!()
        }

        match self {
            Item::Heal => cast_heal(&mut app.objects, &mut app.log),
            Item::Lightning => cast_lightning(
                &mut app.objects,
                &app.gamemap,
                &mut app.log,
                target.unwrap(),
            ),

            // NOTE: logic for equipping items is in the event handler,
            // since removing the equipped item from the inventory requires
            // knowing the index it was stored in
            Item::Equipment => UseResult::Equipped,
        }
    }
}

/// effects of a potion of healing. heals the player
pub fn cast_heal(objects: &mut ObjectMap, log: &mut Log) -> UseResult {
    let fighter = match &objects.get(&PLAYER).unwrap().fighter {
        Some(x) => x,
        None => {
            panic!("trying to cast heal, but target_id does not have a fighter component!")
        }
    };

    if fighter.hp == fighter.max_hp {
        log.add(
            String::from("You are already at full health."),
            Color::default(),
        );
        UseResult::Cancelled
    } else {
        const HEAL_AMOUNT: u16 = 10;
        heal(objects, PLAYER, HEAL_AMOUNT);
        log.add(
            String::from("Your wounds start to close."),
            Color::default(),
        );
        UseResult::UsedUp
    }
}

/// effects of a scroll of lightning. smites a chosen target within line of sight
pub fn cast_lightning(
    objects: &mut ObjectMap,
    gamemap: &GameMap,
    log: &mut Log,
    target: Position,
) -> UseResult {
    // get all fighters within line of sight, minus the player

    // let mut valid_targets = Vec::new();
    // for id in gamemap.object_ids.iter() {
    //     let pos = &objects.get(id).unwrap().pos;
    //     if *id == PLAYER || !gamemap.is_visible(pos.x, pos.y) {
    //         continue;
    //     }
    //
    //     if let Some(_fighter) = &objects.get(id).unwrap().fighter {
    //         valid_targets.push(*id);
    //     }
    // }
    //
    // if valid_targets.len() == 0 {
    //     log.push(format!("No targets in sight."));
    //     return UseResult::Cancelled;
    // }

    // let mut rng = rand::rng();
    // let target_id = valid_targets.choose(&mut rng).unwrap();

    let target_id = match get_blocking_object_id(objects, gamemap, target) {
        Some(x) => {
            if x == PLAYER {
                log.add(String::from("Can't target yourself!"), Color::default());
                return UseResult::Cancelled;
            } else {
                x
            }
        }
        None => {
            log.add(String::from("No targets there."), Color::default());
            return UseResult::Cancelled;
        }
    };

    let fighter = match &objects.get(&target_id).unwrap().fighter {
        Some(x) => x,
        None => {
            panic!("trying to cast lightning, but target_id does not have a fighter component!")
        }
    };

    const LIGHTNING_DAMAGE: i16 = 8;
    let damage = LIGHTNING_DAMAGE - fighter.defense;

    let target_obj = objects.get(&target_id).unwrap();
    let attack_desc = format!("Lightning smites the {}", target_obj.name);

    if damage > 0 {
        take_damage(objects, log, target_id, damage as u16);
        log.add(
            format!("{} for {} damage.", attack_desc, damage),
            Color::LightBlue,
        );
    } else {
        log.add(
            format!("{} but does no damage.", attack_desc),
            Color::default(),
        );
    }

    UseResult::UsedUp
}
