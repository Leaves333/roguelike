use std::{cmp::Ordering, collections::BinaryHeap};

use crate::app::procgen::DungeonConfig;
use rand::Rng;
use ratatui::style::{Color, Style, Stylize};

use crate::{
    app::{Action, App, GameScreen, PLAYER, VIEW_RADIUS},
    components::{AIType, DeathCallback, Item, MELEE_FORGET_TIME, MeleeAIData, Position},
    gamemap::coords_to_idx,
    los,
    pathfinding::Pathfinder,
};

// NOTE: this crate contains functions that control the gameplay
pub enum InputDirection {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

fn direction_to_deltas(direction: InputDirection) -> (i16, i16) {
    match direction {
        InputDirection::Up => (0, -1),
        InputDirection::Down => (0, 1),
        InputDirection::Left => (-1, 0),
        InputDirection::Right => (1, 0),
        InputDirection::UpLeft => (-1, -1),
        InputDirection::UpRight => (1, -1),
        InputDirection::DownLeft => (-1, 1),
        InputDirection::DownRight => (1, 1),
    }
}

/// used to determine if an item was sucessfully used
pub enum UseResult {
    UsedUp,
    Equipped,
    Cancelled,
}

/// different targeting modes for targeted abilities
#[derive(PartialEq, Eq)]
pub enum TargetingMode {
    SmiteEnemy, // smite target any enemy in line of sight
}

/// returns the true power of an fighter, after factoring in bonuses
pub fn power(app: &App, id: usize) -> i16 {
    let obj = app.objects.get(&id).unwrap();

    // return a default of 0 if object has no fighter
    if obj.fighter.is_none() {
        return 0;
    }

    let base_power = obj.fighter.as_ref().unwrap().power;
    let bonus_power: i16 = match id.cmp(&PLAYER) {
        Ordering::Equal => {
            // TODO: equipment calculations
            let mut bonus: i16 = 0;
            for id_option in &app.equipment {
                if id_option.is_none() {
                    continue;
                }

                let obj = app.objects.get(id_option.as_ref().unwrap()).unwrap();
                let equip = obj.equipment.as_ref().unwrap();
                bonus += equip.power_bonus;
            }

            bonus
        }
        _ => 0,
    };

    base_power + bonus_power
}

/// returns the true defense of an fighter, after factoring in bonuses
pub fn defense(app: &App, id: usize) -> i16 {
    let obj = app.objects.get(&id).unwrap();

    // return a default of 0 if object has no fighter
    if obj.fighter.is_none() {
        return 0;
    }

    let base_defense = obj.fighter.as_ref().unwrap().defense;
    let bonus_defense: i16 = match id.cmp(&PLAYER) {
        Ordering::Equal => {
            // TODO: equipment calculations
            let mut bonus: i16 = 0;
            for id_option in &app.equipment {
                if id_option.is_none() {
                    continue;
                }

                let obj = app.objects.get(id_option.as_ref().unwrap()).unwrap();
                let equip = obj.equipment.as_ref().unwrap();
                bonus += equip.defense_bonus;
            }

            bonus
        }
        _ => 0,
    };

    base_defense + bonus_defense
}

/// returns the amount of damage an attack does.
/// note: defense blocks a random amount of damage between def/2 and def
pub fn damage(power: i16, defense: i16) -> i16 {
    let mut rng = rand::rng();
    let mitigated_damage = rng.random_range((defense / 2)..=defense);
    return power.saturating_sub(mitigated_damage).max(0);
}

/// heals an entity for the specified amount
pub fn heal(app: &mut App, id: usize, heal_amount: u16) {
    let obj = app.objects.get_mut(&id).unwrap();
    if let Some(fighter) = obj.fighter.as_mut() {
        fighter.hp += heal_amount;
        fighter.hp = fighter.hp.min(fighter.max_hp)
    }
}

/// applies damage to an entity for the specified amount
pub fn take_damage(app: &mut App, id: usize, damage: u16) {
    let obj = &mut app.objects.get_mut(&id).unwrap();
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
            DeathCallback::Player => player_death(app),
            DeathCallback::Monster => monster_death(app, id),
        }
    }
}

pub fn player_death(app: &mut App) {
    let player = &mut app.objects.get_mut(&PLAYER).unwrap();
    let renderable = &mut player.renderable;
    renderable.glyph = '%';
    renderable.fg = Color::Red;

    app.add_to_log(String::from("You died!"), Style::new().italic().red());
}

pub fn monster_death(app: &mut App, id: usize) {
    let monster = &mut app.objects.get_mut(&id).unwrap();
    let message = format!("{} dies!", monster.name);

    let monster_pos = app.gamemap.get_position(id).unwrap();
    app.gamemap.remove_blocker(monster_pos.x, monster_pos.y);

    // TODO: add blood to the tile after monster death

    // let renderable = &mut monster.renderable;
    // renderable.glyph = '%';
    // renderable.fg = Color::Red;
    //
    // monster.blocks_movement = false;
    // monster.render_layer = RenderLayer::Corpse;
    // monster.alive = false;
    // monster.fighter = None;
    // monster.name = format!("remains of {}", monster.name);

    app.add_to_log(message, Color::Red);
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
            cursor: app.gamemap.get_position(PLAYER).unwrap(),
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
            Item::Heal => cast_heal(app),
            Item::Lightning => cast_lightning(app, target.unwrap()),

            // NOTE: logic for equipping items is in use_item, since removing the equipped item
            // from the inventory requires knowing the index it was stored in
            Item::Equipment => UseResult::Equipped,
        }
    }
}

/// effects of a potion of healing. heals the player
pub fn cast_heal(app: &mut App) -> UseResult {
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
        const HEAL_AMOUNT: u16 = 10;
        heal(app, PLAYER, HEAL_AMOUNT);
        app.add_to_log(
            String::from("Your wounds start to close."),
            Color::default(),
        );
        UseResult::UsedUp
    }
}

/// effects of a scroll of lightning. smites a chosen target within line of sight
pub fn cast_lightning(app: &mut App, target: Position) -> UseResult {
    let target_id = match get_blocking_object_id(app, target.x, target.y) {
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

    const LIGHTNING_DAMAGE: i16 = 8;
    let damage_dealt = damage(LIGHTNING_DAMAGE, defense(app, target_id));

    let target_obj = app.objects.get(&target_id).unwrap();
    let attack_desc = format!("Lightning smites the {}", target_obj.name);

    if damage_dealt > 0 {
        take_damage(app, target_id, damage_dealt as u16);
        app.add_to_log(
            format!("{} for {} damage.", attack_desc, damage_dealt),
            Color::LightBlue,
        );
    } else {
        app.add_to_log(
            format!("{} but does no damage.", attack_desc),
            Color::default(),
        );
    }

    UseResult::UsedUp
}

/// each monster whose next scheduled action is before the current time acts
pub fn handle_monster_turns(app: &mut App) {
    loop {
        let top = app.action_queue.peek();
        let Some(action) = top else {
            return;
        };

        if action.time > app.time {
            return;
        }

        // safe to unwrap here because we checked it was Some earlier
        let action = app.action_queue.pop().unwrap();
        perform_action(app, action);
    }
}

/// performs an action for the specified id
/// and adds it back into the queue
pub fn perform_action(app: &mut App, action: Action) {
    let obj = match app.objects.get(&action.id) {
        None => {
            return;
        }
        Some(x) => x,
    };

    if !obj.alive {
        return;
    }

    let Some(ai_type) = &obj.ai else {
        return;
    };

    let time_taken = match ai_type {
        AIType::Melee(_) => handle_melee_ai(app, action.id),
        AIType::Ranged => {
            todo!()
        }
    };

    app.action_queue.push(Action {
        time: action.time + time_taken,
        id: action.id,
    });
}

/// makes a monster act according to melee ai
/// assumes that said monster has an MeleeAI component
/// returns the amount of time that this monster's turn took
pub fn handle_melee_ai(app: &mut App, id: usize) -> u64 {
    let Some(monster) = app.objects.get_contents().get_mut(&id) else {
        panic!("handle_melee_ai was passed an invalid monster id!")
    };

    let ai_data: &mut MeleeAIData = match &mut monster.ai {
        None => {
            panic!("handle_melee_ai called on object with no AI component!")
        }
        Some(ai_type) => match ai_type {
            AIType::Melee(data) => data,
            _ => {
                panic!("handle_melee_ai called on object with a non-melee AI type!")
            }
        },
    };

    // check if player is in line of sight
    // NOTE: rework los algorithm later, for now assume it is symmetric
    let monster_pos = app.gamemap.get_position(id).unwrap();
    if app.gamemap.is_visible(monster_pos.x, monster_pos.y) {
        ai_data.target = Some(PLAYER);
        ai_data.last_seen_time = Some(app.time);
    }

    // forget the target if we haven't seen it recently
    match ai_data.last_seen_time {
        Some(seen_time) => {
            if seen_time + MELEE_FORGET_TIME <= app.time {
                ai_data.target = None;
            }
        }
        None => {}
    }

    // read these variables here, so we can free the reference to `ai_data`
    let attack_time = ai_data.attack_speed;
    let move_time = ai_data.move_speed;

    let target = match ai_data.target {
        Some(id) => id,
        None => {
            return move_time;
        }
    };

    // find path to the player
    let mut costs = Vec::new();
    costs.resize((app.gamemap.height * app.gamemap.width) as usize, 0);
    for y in 0..app.gamemap.height {
        for x in 0..app.gamemap.width {
            if app.gamemap.get_ref(x, y).is_walkable() {
                costs[coords_to_idx(x, y, app.gamemap.width)] += 1;
            }
        }
    }

    let pathfinder = Pathfinder::new(
        costs,
        (monster_pos.x, monster_pos.y),
        app.gamemap.width,
        app.gamemap.height,
        2,
        3,
    );

    let target_pos = app.gamemap.get_position(target).unwrap();
    let path = pathfinder.path_to((target_pos.x, target_pos.y));

    if path.len() == 0 {
        return 100;
    } else if path.len() == 1 {
        melee_action(app, id, *path.first().unwrap());
        return attack_time;
    } else {
        move_action(app, id, *path.first().unwrap());
        return move_time;
    }
}

/// move an object to (target_x, target_y)
pub fn move_action(app: &mut App, id: usize, (target_x, target_y): (u16, u16)) {
    if !app.gamemap.get_ref(target_x, target_y).is_walkable() {
        return; // destination is blocked by a tile
    }

    if let Some(_) = get_blocking_object_id(app, target_x, target_y) {
        return; // destination is blocked by an object
    }

    let pos = app.gamemap.get_position(id).unwrap();
    let obj = app.gamemap.remove_blocker(pos.x, pos.y);
    app.gamemap.place_blocker(obj, target_x, target_y);

    assert!(obj == id); // sanity check that we got the right object
}

/// returns the amount of time this action took
pub fn melee_action(app: &mut App, attacker_id: usize, (target_x, target_y): (u16, u16)) {
    // check that there is an object to attack
    let target_id = match get_blocking_object_id(app, target_x, target_y) {
        Some(x) => x,
        None => {
            return; // should never hit this case
        }
    };

    let attacker_power = power(&app, attacker_id);
    let target_defense = defense(&app, target_id);
    let damage = (attacker_power - target_defense).max(0) as u16;

    let [Some(attacker), Some(target)] = app
        .objects
        .get_contents()
        .get_disjoint_mut([&attacker_id, &target_id])
    else {
        panic!("invalid ids passed to melee_action()!");
    };

    let attack_desc = format!("{} attacks {}", attacker.name, target.name);
    if damage > 0 {
        take_damage(app, target_id, damage);
        app.add_to_log(
            format!("{} for {} damage.", attack_desc, damage),
            Color::default(),
        );
    } else {
        app.add_to_log(
            format!("{} but does no damage.", attack_desc),
            Color::default(),
        );
    }
}

pub fn bump_action(app: &mut App, id: usize, direction: InputDirection) {
    // check that action target is in bounds
    let pos = app.gamemap.get_position(id).unwrap();
    let deltas = direction_to_deltas(direction);
    let (dx, dy) = deltas;
    if !app.gamemap.in_bounds(pos.x as i16 + dx, pos.y as i16 + dy) {
        return; // destination is not in bounds
    }
    let (target_x, target_y) = ((pos.x as i16 + dx) as u16, (pos.y as i16 + dy) as u16);

    // decide which action to take
    match get_blocking_object_id(app, target_x, target_y) {
        Some(_) => {
            melee_action(app, id, (target_x, target_y));
        }
        None => {
            move_action(app, id, (target_x, target_y));
        }
    };
}

pub fn get_blocking_object_id(app: &App, x: u16, y: u16) -> Option<usize> {
    // for id in app.gamemap.object_ids.iter() {
    //     let obj = &app.objects.get(id).unwrap();
    //     if obj.blocks_movement && obj.pos.x == x && obj.pos.y == y {
    //         return Some(id.clone());
    //     }
    // }
    app.gamemap.get_ref(x, y).blocker
}

// recompute visible area based on the player's fov
pub fn update_fov(app: &mut App, radius: u16) {
    // TODO: use a different symmetric algo to calculate line of sight

    let position = app.gamemap.get_position(PLAYER).unwrap();
    let (player_x, player_y) = (position.x, position.y);

    app.gamemap.visible.fill(false);

    // calculate bounds for visibility
    let (xlow, xhigh) = (
        (player_x.saturating_sub(radius)).max(0),
        (player_x + radius).min(app.gamemap.width - 1),
    );
    let (ylow, yhigh) = (
        (player_y.saturating_sub(radius)).max(0),
        (player_y + radius).min(app.gamemap.height - 1),
    );

    // loop through each x, y to check visibility
    for target_x in xlow..=xhigh {
        for target_y in ylow..=yhigh {
            // calculate los path from player to target square
            let path: Vec<(u16, u16)> = los::bresenham(
                (player_x.into(), player_y.into()),
                (target_x.into(), target_y.into()),
            )
            .iter()
            .map(|&(x, y)| (x as u16, y as u16))
            .collect();

            // walk along the path to check for visibility
            for (x, y) in path {
                if !app.gamemap.get_ref(x, y).is_transparent() {
                    app.gamemap.set_visible(x, y, true);
                    break;
                }
                app.gamemap.set_visible(x, y, true);
            }
        }
    }

    // explored |= visible
    for (e, &v) in app
        .gamemap
        .explored
        .iter_mut()
        .zip(app.gamemap.visible.iter())
    {
        *e |= v;
    }

    // for each visible tile, update the renderable it was last seen as
    for x in xlow..=xhigh {
        for y in ylow..=yhigh {
            if app.gamemap.is_visible(x, y) {
                let tile = app.gamemap.get_ref(x, y);
                app.gamemap.set_last_seen(
                    x,
                    y,
                    crate::app::render::tile_topmost_renderable(app, tile),
                );
            }
        }
    }
}

/// attempts to go down stairs at the current location.
/// returns true if successful, false if not
pub fn go_down_stairs(app: &mut App) -> bool {
    let player_pos = app.gamemap.get_position(PLAYER).unwrap();

    // match for objects at player_pos
    // let objects_at_pos: Vec<&Object> = app
    //     .gamemap
    //     .object_ids
    //     .iter()
    //     .map(|id| app.objects.get(id).unwrap())
    //     .filter(|&obj| obj.pos == player_pos)
    //     .collect();
    //
    // let on_stairs = objects_at_pos
    //     .iter()
    //     .filter(|&obj| obj.name == "Stairs")
    //     .count()
    //     > 0;

    let on_stairs = {
        let item = app.gamemap.get_ref(player_pos.x, player_pos.y).item;
        if let Some(id) = item {
            let item = app.objects.get(&id).unwrap();
            item.name == "Stairs"
        } else {
            false
        }
    };

    if !on_stairs {
        app.add_to_log("Can't go down, not standing on stairs.", Color::default());
        return false;
    }

    // clear the action queue, so enemies from the previous floor stop taking actions
    app.action_queue = BinaryHeap::new();

    // NOTE: code to generate next stage
    let cur_level = app.gamemap.level;
    app.generate_dungeon(DungeonConfig::default().set_level(cur_level + 1));
    app.add_to_log(
        "As you dive deeper into the dungeon, you find a moment to rest and recover.",
        Color::Magenta,
    );
    app.add_to_log("You feel stronger.", Color::Magenta);
    update_fov(app, VIEW_RADIUS);

    let player_fighter = app
        .objects
        .get_mut(&PLAYER)
        .unwrap()
        .fighter
        .as_mut()
        .unwrap();
    player_fighter.max_hp += 5;
    player_fighter.hp = player_fighter.max_hp;

    true
}
