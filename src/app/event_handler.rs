use core::panic;
use std::collections::HashSet;

use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use rand::seq::IndexedRandom;
use ratatui::DefaultTerminal;
use ratatui::style::Color;

use crate::components::{AIType, DeathCallback, Item, RenderStatus};
use crate::gamemap::coords_to_idx;
use crate::los;
use crate::pathfinding::Pathfinder;

use super::render::GameScreen;
use super::{App, PLAYER};

enum InputDirection {
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

/// used to determine if the player took a turn or not
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

/// used to determine if an item was sucessfully used
pub enum UseResult {
    UsedUp,
    Cancelled,
}

/// mutably borrow two *separate* elements from the given slice.
/// panics when the indexes are equal or out of bounds.
/// code from [https://tomassedovic.github.io/roguelike-tutorial/part-6-going-berserk.html]
fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split_at_index = std::cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}

impl App {
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()? {
                let action = self.handle_keys(key);
                match action {
                    PlayerAction::TookTurn => {
                        // monsters act...
                        self.handle_monster_turns();

                        // update fov
                        let view_radius = 8;
                        self.update_fov(view_radius);

                        // self.log.push(String::from("### new turn"));
                    }
                    PlayerAction::DidntTakeTurn => {
                        // nothing happens
                    }
                    PlayerAction::Exit => {
                        break Ok(());
                    }
                }
            }
        }
    }

    /// translate the key event into the appropriate gameplay actions
    fn handle_keys(&mut self, key: KeyEvent) -> PlayerAction {
        // match generic keybinds, used for menu navigation
        // NOTE: these need to be handled first
        match key.modifiers {
            KeyModifiers::CONTROL => match key.code {
                KeyCode::Char('l') => {
                    self.toggle_fullscreen_log();
                    return PlayerAction::DidntTakeTurn;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    return PlayerAction::Exit;
                }
                _ => {}
            },
            _ => match key.code {
                KeyCode::Esc => {
                    self.switch_to_main_screen();
                    return PlayerAction::DidntTakeTurn;
                }
                _ => {}
            },
        };

        // keybinds specific to certain gamescreens
        match self.game_screen {
            GameScreen::Main => {
                match key.code {
                    // movement keys
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.bump_action(PLAYER, InputDirection::Right);
                        return PlayerAction::TookTurn;
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.bump_action(PLAYER, InputDirection::Left);
                        return PlayerAction::TookTurn;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.bump_action(PLAYER, InputDirection::Down);
                        return PlayerAction::TookTurn;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.bump_action(PLAYER, InputDirection::Up);
                        return PlayerAction::TookTurn;
                    }
                    KeyCode::Char('u') => {
                        self.bump_action(PLAYER, InputDirection::UpRight);
                        return PlayerAction::TookTurn;
                    }
                    KeyCode::Char('y') => {
                        self.bump_action(PLAYER, InputDirection::UpLeft);
                        return PlayerAction::TookTurn;
                    }
                    KeyCode::Char('n') => {
                        self.bump_action(PLAYER, InputDirection::DownRight);
                        return PlayerAction::TookTurn;
                    }
                    KeyCode::Char('b') => {
                        self.bump_action(PLAYER, InputDirection::DownLeft);
                        return PlayerAction::TookTurn;
                    }
                    KeyCode::Char('.') => {
                        // wait action, nothing is done
                        return PlayerAction::TookTurn;
                    }

                    // inventory keys: '1' to '9' and '0'
                    KeyCode::Char(c @ '1'..='9') | KeyCode::Char(c @ '0') => {
                        let index = match c {
                            '1'..='9' => c as usize - '1' as usize,
                            '0' => 9,
                            _ => unreachable!(),
                        };

                        if self.inventory.len() > index {
                            let use_result = self.use_item(index);
                            return match use_result {
                                UseResult::UsedUp => PlayerAction::TookTurn,
                                UseResult::Cancelled => PlayerAction::DidntTakeTurn,
                            };
                        }
                    }

                    // can only go to examine mode on main game screen
                    KeyCode::Char('x') => {
                        self.toggle_examine_mode();
                        return PlayerAction::DidntTakeTurn;
                    }

                    // actual controls lol
                    KeyCode::Char('g') => {
                        // pick up the first item at location
                        let player_pos = &self.objects.get(&PLAYER).unwrap().pos;
                        let id = self.gamemap.object_ids.iter().find(|x| {
                            let obj = &self.objects.get(x).unwrap();
                            obj.pos.x == player_pos.x
                                && obj.pos.y == player_pos.y
                                && obj.item.is_some()
                        });
                        match id {
                            Some(id) => {
                                self.pick_item_up(id.clone());
                                return PlayerAction::TookTurn;
                            }
                            None => {
                                return PlayerAction::DidntTakeTurn;
                            }
                        }
                    }
                    _ => {}
                }
            }
            GameScreen::Log { ref mut offset } => match key.code {
                KeyCode::PageUp => {
                    *offset += 10;
                }
                KeyCode::PageDown => {
                    *offset = offset.saturating_sub(10);
                }
                KeyCode::Char('k') => {
                    *offset += 1;
                }
                KeyCode::Char('j') => {
                    *offset = offset.saturating_sub(1);
                }
                _ => {}
            },
            GameScreen::Examine { ref mut cursor } => match key.code {
                // move cursor around
                // do checks to keep cursor within bounds of the gamemap here
                KeyCode::Down | KeyCode::Char('j') => {
                    cursor.y = (cursor.y + 1).min(self.gamemap.height - 1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    cursor.y = cursor.y.saturating_sub(1);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    cursor.x = (cursor.x + 1).min(self.gamemap.width - 1);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    cursor.x = cursor.x.saturating_sub(1);
                }

                // exit examine mode
                KeyCode::Char('x') => {
                    self.toggle_examine_mode();
                    return PlayerAction::DidntTakeTurn;
                }
                _ => {}
            },
        };

        // if no existing keybinds were matched
        return PlayerAction::DidntTakeTurn;
    }

    fn toggle_fullscreen_log(&mut self) {
        match self.game_screen {
            GameScreen::Log { offset: _ } => self.game_screen = GameScreen::Main,
            _ => self.game_screen = GameScreen::Log { offset: 0 },
        }
    }

    fn toggle_examine_mode(&mut self) {
        match self.game_screen {
            GameScreen::Examine { cursor: _ } => self.game_screen = GameScreen::Main,
            _ => {
                // set default cursor location to player's position
                self.game_screen = GameScreen::Examine {
                    cursor: { self.objects.get(&PLAYER).unwrap().pos.clone() },
                }
            }
        }
    }

    fn switch_to_main_screen(&mut self) {
        self.game_screen = GameScreen::Main;
    }

    /// makes all the monsters take a turn
    fn handle_monster_turns(&mut self) {
        for id in self.gamemap.object_ids.clone().iter() {
            let obj = match self.objects.get(id) {
                None => {
                    continue;
                }
                Some(x) => x,
            };

            if !obj.alive {
                continue;
            }

            if let Some(ai_type) = &obj.ai {
                match ai_type {
                    AIType::Melee => {
                        self.handle_melee_ai(id.clone());
                    }
                }
            }
        }
    }

    /// makes a monster act according to melee ai
    fn handle_melee_ai(&mut self, id: usize) {
        let [Some(player), Some(monster)] = self.objects.get_disjoint_mut([&PLAYER, &id]) else {
            panic!("invalid ids while handling melee ai!")
        };

        let out_of_range =
            monster.pos.x.abs_diff(player.pos.x) > 8 || monster.pos.y.abs_diff(player.pos.y) > 8;

        if out_of_range {
            return;
        }

        // NOTE: rework los algorithm later, for now assume it is symmetric
        if !self.gamemap.is_visible(monster.pos.x, monster.pos.y) {
            return;
        }

        // find path to the player
        let mut costs = Vec::new();
        costs.resize((self.gamemap.height * self.gamemap.width) as usize, 0);
        for y in 0..self.gamemap.height {
            for x in 0..self.gamemap.width {
                if self.gamemap.get_ref(x, y).walkable {
                    costs[coords_to_idx(x, y, self.gamemap.width)] += 1;
                }
            }
        }

        let pathfinder = Pathfinder::new(
            costs,
            (monster.pos.x, monster.pos.y),
            self.gamemap.width,
            self.gamemap.height,
            2,
            3,
        );

        let path = pathfinder.path_to((player.pos.x, player.pos.y));
        if path.len() == 0 {
            self.log
                .push(format!("{} just sits and waits.", monster.name));
        } else if path.len() == 1 {
            self.melee_action(id, *path.first().unwrap());
        } else {
            self.log
                .push(format!("{} moves towards the player!", monster.name));
            self.move_action(id, *path.first().unwrap());
        }
    }

    /// move an object to (target_x, target_y)
    fn move_action(&mut self, id: usize, (target_x, target_y): (u16, u16)) {
        if !self.gamemap.get_ref(target_x, target_y).walkable {
            return; // destination is blocked by a tile
        }

        if let Some(_) = self.get_blocking_object_id(target_x, target_y) {
            return; // destination is blocked by an object
        }

        let pos = &mut self.objects.get_mut(&id).unwrap().pos;
        pos.x = target_x;
        pos.y = target_y;
    }

    fn melee_action(&mut self, attacker_id: usize, (target_x, target_y): (u16, u16)) {
        // check that there is an object to attack
        let target_id = match self.get_blocking_object_id(target_x, target_y) {
            Some(x) => x,
            None => {
                return; // should never hit this case
            }
        };

        let [Some(attacker), Some(target)] =
            self.objects.get_disjoint_mut([&attacker_id, &target_id])
        else {
            panic!("invalid ids passed to melee_action()!");
        };

        let attacker_fighter = &attacker.fighter.as_ref().unwrap();
        let target_fighter = &mut target.fighter.as_mut().unwrap();

        let damage = (attacker_fighter.power - target_fighter.defense).max(0) as u16;
        let attack_desc = format!("{} attacks {}", attacker.name, target.name);

        if damage > 0 {
            self.take_damage(target_id, damage);
            self.log
                .push(format!("{} for {} damage.", attack_desc, damage));
        } else {
            self.log
                .push(format!("{} but does no damage.", attack_desc));
        }
    }

    fn bump_action(&mut self, id: usize, direction: InputDirection) {
        // check that action target is in bounds
        let pos = &self.objects.get(&id).unwrap().pos;
        let deltas = direction_to_deltas(direction);
        let (dx, dy) = deltas;
        if !self.gamemap.in_bounds(pos.x as i16 + dx, pos.y as i16 + dy) {
            return; // destination is not in bounds
        }
        let (target_x, target_y) = ((pos.x as i16 + dx) as u16, (pos.y as i16 + dy) as u16);

        // decide which action to take
        match self.get_blocking_object_id(target_x, target_y) {
            Some(_) => {
                self.melee_action(id, (target_x, target_y));
            }
            None => {
                self.move_action(id, (target_x, target_y));
            }
        };
    }

    fn take_damage(&mut self, id: usize, damage: u16) {
        // apply damage if possible
        let obj = &mut self.objects.get_mut(&id).unwrap();
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
                DeathCallback::Player => self.player_death(),
                DeathCallback::Monster => self.monster_death(id),
            }
        }
    }

    fn heal(&mut self, id: usize, heal_amount: u16) {
        let obj = &mut self.objects.get_mut(&id).unwrap();
        if let Some(fighter) = obj.fighter.as_mut() {
            fighter.hp += heal_amount;
            fighter.hp = fighter.hp.min(fighter.max_hp)
        }
    }

    fn player_death(&mut self) {
        let player = &mut self.objects.get_mut(&PLAYER).unwrap();
        self.log.push(String::from("you died!"));

        let renderable = &mut player.renderable;
        renderable.glyph = '%';
        renderable.fg = Color::Red;
    }

    fn monster_death(&mut self, id: usize) {
        let monster = &mut self.objects.get_mut(&id).unwrap();
        self.log.push(format!("{} dies!", monster.name));

        let renderable = &mut monster.renderable;
        renderable.glyph = '%';
        renderable.fg = Color::Red;

        monster.blocks_movement = false;
        monster.alive = false;
        monster.fighter = None;
        monster.name = format!("remains of {}", monster.name);
    }

    pub fn get_blocking_object_id(&self, x: u16, y: u16) -> Option<usize> {
        for id in self.gamemap.object_ids.iter() {
            let obj = &self.objects.get(id).unwrap();
            if obj.blocks_movement && obj.pos.x == x && obj.pos.y == y {
                return Some(id.clone());
            }
        }
        return None;
    }

    // recompute visible area based on the player's fov
    pub fn update_fov(&mut self, radius: u16) {
        // TODO: use a different symmetric algo to calculate line of sight

        let position = &self.objects.get(&PLAYER).unwrap().pos;
        let (player_x, player_y) = (position.x, position.y);

        self.gamemap.visible.fill(false);

        // calculate bounds for visibility
        let (xlow, xhigh) = (
            (player_x.saturating_sub(radius)).max(0),
            (player_x + radius).min(self.gamemap.width - 1),
        );
        let (ylow, yhigh) = (
            (player_y.saturating_sub(radius)).max(0),
            (player_y + radius).min(self.gamemap.width - 1),
        );

        // loop through each x, y to check visibility
        let mut visited = HashSet::new();
        for target_x in xlow..=xhigh {
            for target_y in ylow..=yhigh {
                // already checked this square
                if visited.contains(&(target_x, target_y)) {
                    continue;
                }

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
                    visited.insert((x, y));
                    if !self.gamemap.get_ref(x, y).transparent {
                        self.gamemap.set_visible(x, y, true);
                        break;
                    }
                    self.gamemap.set_visible(x, y, true);
                }
            }
        }

        // explored |= visible
        for (e, &v) in self
            .gamemap
            .explored
            .iter_mut()
            .zip(self.gamemap.visible.iter())
        {
            *e |= v;
        }
    }

    /// moves and item from the gamemap into the player inventory based on object id
    fn pick_item_up(&mut self, id: usize) {
        if self.inventory.len() >= 10 {
            self.log.push(format!("Cannot hold that many items."));
        } else {
            let idx = self.gamemap.object_ids.iter().position(|&x| x == id);
            match idx {
                Some(x) => {
                    // add the item to the inventory
                    let item_id = self.gamemap.object_ids.swap_remove(x);
                    self.inventory.push(item_id);

                    // hide it on the map
                    let item_obj = self.objects.get_mut(&item_id).unwrap();
                    item_obj.render_status = RenderStatus::Hide;

                    self.log.push(format!("Picked up {}.", item_obj.name));
                }
                None => {
                    panic!("invalid object id passed to pick_item_up()!")
                }
            }
        }
    }

    /// uses an item from the specified index in the inventory
    fn use_item(&mut self, inventory_idx: usize) -> UseResult {
        let item_id = self.inventory[inventory_idx];
        let item = match &self.objects.get(&item_id).unwrap().item {
            Some(x) => x,
            None => {
                panic!("use_item called with an object without an item component!")
            }
        };

        let on_use = match item {
            Item::Heal => self.cast_heal(),
            Item::Lightning => self.cast_lightning(),
        };

        match on_use {
            UseResult::UsedUp => {
                // delete item after being used
                self.inventory.remove(inventory_idx);
            }
            UseResult::Cancelled => {
                // item wasn't used, don't delete it
            }
        };

        on_use
    }

    /// effects of a potion of healing. heals the player for 4 hp
    pub fn cast_heal(&mut self) -> UseResult {
        let fighter = match &self.objects.get(&PLAYER).unwrap().fighter {
            Some(x) => x,
            None => {
                panic!("trying to cast heal, but target_id does not have a fighter component!")
            }
        };

        if fighter.hp == fighter.max_hp {
            self.log
                .push(String::from("You are already at full health."));
            UseResult::Cancelled
        } else {
            self.heal(PLAYER, 4);
            self.log.push(String::from("Your wounds start to close."));
            UseResult::UsedUp
        }
    }

    /// effects of a scroll of lightning. randomly smites a target within line of sight
    pub fn cast_lightning(&mut self) -> UseResult {
        // get all fighters within line of sight, minus the player
        let mut valid_targets = Vec::new();
        for id in self.gamemap.object_ids.iter() {
            let pos = &self.objects.get(id).unwrap().pos;
            if *id == PLAYER || !self.gamemap.is_visible(pos.x, pos.y) {
                continue;
            }

            if let Some(_fighter) = &self.objects.get(id).unwrap().fighter {
                valid_targets.push(*id);
            }
        }

        if valid_targets.len() == 0 {
            self.log.push(format!("No targets in sight."));
            return UseResult::Cancelled;
        }

        let mut rng = rand::rng();
        let target_id = valid_targets.choose(&mut rng).unwrap();
        let fighter = match &self.objects.get(target_id).unwrap().fighter {
            Some(x) => x,
            None => {
                panic!("trying to cast lightning, but target_id does not have a fighter component!")
            }
        };

        const LIGHTNING_DAMAGE: i16 = 8;
        let damage = LIGHTNING_DAMAGE - fighter.defense;

        let target_obj = self.objects.get(target_id).unwrap();
        let attack_desc = format!("Lightning smites the {}", target_obj.name);

        if damage > 0 {
            self.take_damage(*target_id, damage as u16);
            self.log
                .push(format!("{} for {} damage.", attack_desc, damage));
        } else {
            self.log
                .push(format!("{} but does no damage.", attack_desc));
        }

        UseResult::UsedUp
    }
}
