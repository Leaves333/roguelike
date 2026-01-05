use color_eyre::{Result, eyre::Ok};
use core::panic;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::style::Color;
use std::collections::BinaryHeap;

use crate::app::Action;
use crate::components::{AIType, MELEE_FORGET_TIME, MeleeAIData, Object, SLOT_ORDERING};
use crate::engine::{UseResult, defense, power, take_damage};
use crate::gamemap::coords_to_idx;
use crate::inventory;
use crate::los;
use crate::pathfinding::Pathfinder;

use super::procgen::DungeonConfig;
use super::{App, GameScreen, INVENTORY_SIZE, PLAYER, VIEW_RADIUS};

// NOTE: i want this file to contain logic for handling player controls

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

/// represents the kind of action that a player took
/// NOTE: the Exit variant is here because it impacts the main game loop
/// other actions that only change the state of the app but don't affect the main loop
/// should be handled locally, and not set as a separate enum
enum PlayerAction {
    /// the player took a turn, and their action took u64 time
    TookTurn(u64),
    /// the player didn't take a turn, and we shouldn't increment the time at all
    /// this variant exists to make code more readable
    NoTimeTaken,
    Exit,
}

const PLAYER_MOVEMENT_TIME: u64 = 100;
const PLAYER_ITEM_USE_TIME: u64 = 50;

/// match generic keybinds, used for menu navigation
/// returns a PlayerAction if a keybind was succesfully matched, or None otherwise
fn match_menu_keys(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    match key.modifiers {
        KeyModifiers::CONTROL => match key.code {
            KeyCode::Char('l') => {
                app.toggle_fullscreen_log();
                return Some(PlayerAction::TookTurn(0));
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                return Some(PlayerAction::Exit);
            }
            _ => {}
        },
        _ => match key.code {
            KeyCode::Esc => {
                app.switch_to_main_screen();
                return Some(PlayerAction::TookTurn(0));
            }
            _ => {}
        },
    };
    return None;
}

/// match keybinds for movement
/// returns a PlayerAction if a keybind was succesfully matched, or None otherwise
fn match_movement_keys(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    // movement related controls
    match app.game_screen {
        GameScreen::Main => match key.code {
            // movement keys during the main screen
            KeyCode::Right | KeyCode::Char('l') => {
                app.bump_action(PLAYER, InputDirection::Right);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.bump_action(PLAYER, InputDirection::Left);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.bump_action(PLAYER, InputDirection::Down);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.bump_action(PLAYER, InputDirection::Up);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('u') => {
                app.bump_action(PLAYER, InputDirection::UpRight);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('y') => {
                app.bump_action(PLAYER, InputDirection::UpLeft);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('n') => {
                app.bump_action(PLAYER, InputDirection::DownRight);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('b') => {
                app.bump_action(PLAYER, InputDirection::DownLeft);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('.') => {
                // wait action, nothing is done
                // NOTE: default wait time is 100, independent of player movement speed
                return Some(PlayerAction::TookTurn(100));
            }
            _ => {}
        },
        GameScreen::Examine { ref mut cursor } | GameScreen::Targeting { ref mut cursor, .. } => {
            match key.code {
                // move cursor around during examine and targeting modes
                // do checks to keep cursor within bounds of the gamemap here
                KeyCode::Down | KeyCode::Char('j') => {
                    cursor.y = (cursor.y + 1).min(app.gamemap.height - 1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    cursor.y = cursor.y.saturating_sub(1);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    cursor.x = (cursor.x + 1).min(app.gamemap.width - 1);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    cursor.x = cursor.x.saturating_sub(1);
                }

                KeyCode::Char('u') => {
                    cursor.x = (cursor.x + 1).min(app.gamemap.width - 1);
                    cursor.y = cursor.y.saturating_sub(1);
                }
                KeyCode::Char('y') => {
                    cursor.x = cursor.x.saturating_sub(1);
                    cursor.y = cursor.y.saturating_sub(1);
                }
                KeyCode::Char('n') => {
                    cursor.x = (cursor.x + 1).min(app.gamemap.width - 1);
                    cursor.y = (cursor.y + 1).min(app.gamemap.height - 1);
                }
                KeyCode::Char('b') => {
                    cursor.x = cursor.x.saturating_sub(1);
                    cursor.y = (cursor.y + 1).min(app.gamemap.height - 1);
                }
                _ => {}
            }
        }
        _ => {}
    };
    return None;
}

/// matches controls on the main menu
/// returns a PlayerAction if a keybind was succesfully matched, or None otherwise
fn match_main_menu_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    // check we are on the menu screen
    if app.game_screen != GameScreen::Menu {
        return None;
    }

    match key.code {
        KeyCode::Char('n') => {
            // start new game
            app.new_game();
            app.switch_to_main_screen();
            Some(PlayerAction::NoTimeTaken)
        }
        KeyCode::Char('l') => {
            // loads an existing game from a save file
            let _ = app.load_game();
            app.switch_to_main_screen();
            Some(PlayerAction::NoTimeTaken)
        }
        KeyCode::Char('q') => {
            // quit the game
            Some(PlayerAction::Exit)
        }
        _ => None,
    }
}

fn match_inventory_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    if app.game_screen != GameScreen::Main {
        return None;
    }

    // use alt-number to drop item from inventory
    match key.modifiers {
        KeyModifiers::ALT => {
            match key.code {
                // drop item from inventory
                KeyCode::Char(c @ '1'..='9') | KeyCode::Char(c @ '0') => {
                    let index = match c {
                        '1'..='9' => c as usize - '1' as usize,
                        '0' => 9,
                        _ => unreachable!(),
                    };
                    inventory::drop_item(app, index);
                }
                _ => {}
            }
        }
        _ => {}
    }

    match key.code {
        // number keys to use item from inventory
        KeyCode::Char(c @ '1'..='9') | KeyCode::Char(c @ '0') => {
            let index = match c {
                '1'..='9' => c as usize - '1' as usize,
                '0' => 9,
                _ => unreachable!(),
            };

            if app.inventory.len() > index {
                let item = inventory::get_item_in_inventory(app, index).clone();

                if item.needs_targeting() {
                    // item needs targeting, switch to targeting mode
                    item.on_targeting(app, index);
                    return Some(PlayerAction::NoTimeTaken);
                } else {
                    // item can be used directly
                    let use_result = inventory::use_item(app, index, None);
                    return match use_result {
                        UseResult::UsedUp => Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME)),
                        UseResult::Equipped => Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME)),
                        UseResult::Cancelled => Some(PlayerAction::NoTimeTaken),
                    };
                }
            }
        }

        // unequip item from equipment
        KeyCode::Char(c @ 'A'..='C') => {
            let index = c as usize - 'A' as usize;
            match app.equipment[index] {
                Some(id) => {
                    // check we have enough space in inventory to unequip the item
                    if app.inventory.len() >= INVENTORY_SIZE {
                        app.add_to_log("Not enough space in inventory.", Color::default());
                        return Some(PlayerAction::NoTimeTaken);
                    }

                    // unequip and move to inventory
                    app.inventory.push(id);
                    app.equipment[index] = None;
                    return Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME));
                }
                None => {
                    app.add_to_log(
                        format!("No item equipped on {}.", SLOT_ORDERING[index]),
                        Color::default(),
                    );
                    return Some(PlayerAction::NoTimeTaken);
                }
            }
        }

        // `g`rab the first item at player's location
        KeyCode::Char('g') => {
            let player_pos = &app.objects.get(&PLAYER).unwrap().pos;
            let id = app.gamemap.object_ids.iter().find(|x| {
                let obj = &app.objects.get(x).unwrap();
                obj.pos.x == player_pos.x && obj.pos.y == player_pos.y && obj.item.is_some()
            });
            match id {
                Some(id) => {
                    inventory::pick_item_up(app, id.clone());
                    return Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME));
                }
                None => {
                    return Some(PlayerAction::TookTurn(0));
                }
            }
        }
        _ => {}
    }

    return None;
}

/// matches any remaining game controls on the main screen
fn match_misc_game_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    if app.game_screen != GameScreen::Main {
        return None;
    }

    match key.code {
        // move to examine mode
        KeyCode::Char('x') => {
            app.toggle_examine_mode();
            Some(PlayerAction::NoTimeTaken)
        }

        // go down stairs if stairs exist
        KeyCode::Char('>') => {
            let _ = app.go_down_stairs();
            app.switch_to_main_screen();
            Some(PlayerAction::NoTimeTaken)
        }
        _ => None,
    }
}

fn match_log_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    match app.game_screen {
        GameScreen::Log { ref mut offset } => match key.code {
            KeyCode::PageUp => {
                *offset += 10;
                Some(PlayerAction::NoTimeTaken)
            }
            KeyCode::PageDown => {
                *offset = offset.saturating_sub(10);
                Some(PlayerAction::NoTimeTaken)
            }
            KeyCode::Char('k') => {
                *offset += 1;
                Some(PlayerAction::NoTimeTaken)
            }
            KeyCode::Char('j') => {
                *offset = offset.saturating_sub(1);
                Some(PlayerAction::NoTimeTaken)
            }
            _ => None,
        },
        _ => None,
    }
}

fn match_examine_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    // NOTE: controls for moving the cursor fall under movement controls
    match app.game_screen {
        GameScreen::Examine { .. } => match key.code {
            // exit examine mode
            KeyCode::Char('x') => {
                app.toggle_examine_mode();
                Some(PlayerAction::NoTimeTaken)
            }
            _ => None,
        },
        _ => None,
    }
}

fn match_targeting_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    match app.game_screen {
        GameScreen::Targeting {
            ref cursor,
            inventory_idx,
            ..
        } => match key.code {
            KeyCode::Enter => {
                // use the item and exit targeting mode
                let use_result = inventory::use_item(app, inventory_idx, Some(cursor.clone()));
                app.game_screen = GameScreen::Main;

                match use_result {
                    UseResult::UsedUp => Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME)),
                    UseResult::Equipped => Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME)),
                    UseResult::Cancelled => Some(PlayerAction::NoTimeTaken),
                }
            }
            _ => None,
        },
        _ => None,
    }
}

impl App {
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()? {
                let action = self.handle_keys(key);
                match action {
                    PlayerAction::TookTurn(time_taken) => {
                        if time_taken == 0 {
                            continue;
                        }

                        self.time += time_taken;
                        self.handle_monster_turns();
                        self.update_fov(VIEW_RADIUS);
                    }
                    PlayerAction::NoTimeTaken => {
                        continue;
                    }
                    PlayerAction::Exit => {
                        self.save_game()?;
                        break Ok(());
                    }
                }
            }
        }
    }

    /// translate the key event into the appropriate gameplay actions
    fn handle_keys(&mut self, key: KeyEvent) -> PlayerAction {
        let handlers = &[
            match_menu_keys,
            match_movement_keys,
            match_main_menu_controls,
            match_misc_game_controls,
            match_inventory_controls,
            match_log_controls,
            match_examine_controls,
            match_targeting_controls,
        ];

        // iterates through handlers, and gives the first one with a non-none result
        handlers
            .iter()
            .find_map(|handler| handler(self, key))
            .unwrap_or(PlayerAction::NoTimeTaken)
    }

    pub fn new_game(&mut self) {
        self.generate_dungeon(DungeonConfig::default());
        self.update_fov(VIEW_RADIUS);
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

    /// each monster whose next scheduled action is before the current time acts
    fn handle_monster_turns(&mut self) {
        loop {
            let top = self.action_queue.peek();
            let Some(action) = top else {
                return;
            };

            if action.time > self.time {
                return;
            }

            // safe to unwrap here because we checked it was Some earlier
            let action = self.action_queue.pop().unwrap();
            self.perform_action(action);
        }
    }

    /// performs an action for the specified id
    /// and adds it back into the queue
    fn perform_action(&mut self, action: Action) {
        let obj = match self.objects.get(&action.id) {
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
            AIType::Melee(_) => self.handle_melee_ai(action.id),
            AIType::Ranged => {
                todo!()
            }
        };

        self.action_queue.push(Action {
            time: action.time + time_taken,
            id: action.id,
        });
    }

    /// makes a monster act according to melee ai
    /// assumes that said monster has an MeleeAI component
    /// returns the amount of time that this monster's turn took
    fn handle_melee_ai(&mut self, id: usize) -> u64 {
        let Some(monster) = self.objects.get_contents().get_mut(&id) else {
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
        if self.gamemap.is_visible(monster.pos.x, monster.pos.y) {
            ai_data.target = Some(PLAYER);
            ai_data.last_seen_time = Some(self.time);
        }

        // forget the target if we haven't seen it recently
        match ai_data.last_seen_time {
            Some(seen_time) => {
                if seen_time + MELEE_FORGET_TIME <= self.time {
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

        let monster_name = monster.name.clone();
        let Some(target) = self.objects.get(&target) else {
            panic!("handle_melee_ai: melee AI on {monster_name} had an invalid target id!",);
        };

        let path = pathfinder.path_to((target.pos.x, target.pos.y));
        if path.len() == 0 {
            return 100;
        } else if path.len() == 1 {
            self.melee_action(id, *path.first().unwrap());
            return attack_time;
        } else {
            self.move_action(id, *path.first().unwrap());
            return move_time;
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

    /// returns the amount of time this action took
    fn melee_action(&mut self, attacker_id: usize, (target_x, target_y): (u16, u16)) {
        // check that there is an object to attack
        let target_id = match self.get_blocking_object_id(target_x, target_y) {
            Some(x) => x,
            None => {
                return; // should never hit this case
            }
        };

        let attacker_power = power(&self, attacker_id);
        let target_defense = defense(&self, target_id);
        let damage = (attacker_power - target_defense).max(0) as u16;

        let [Some(attacker), Some(target)] = self
            .objects
            .get_contents()
            .get_disjoint_mut([&attacker_id, &target_id])
        else {
            panic!("invalid ids passed to melee_action()!");
        };

        let attack_desc = format!("{} attacks {}", attacker.name, target.name);
        if damage > 0 {
            take_damage(self, target_id, damage);
            self.add_to_log(
                format!("{} for {} damage.", attack_desc, damage),
                Color::default(),
            );
        } else {
            self.add_to_log(
                format!("{} but does no damage.", attack_desc),
                Color::default(),
            );
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

    /// attempts to go down stairs at the current location.
    /// returns true if successful, false if not
    fn go_down_stairs(&mut self) -> bool {
        let player_pos = self.objects.get(&PLAYER).unwrap().pos;

        // match for objects at player_pos
        let objects_at_pos: Vec<&Object> = self
            .gamemap
            .object_ids
            .iter()
            .map(|id| self.objects.get(id).unwrap())
            .filter(|&obj| obj.pos == player_pos)
            .collect();

        let on_stairs = objects_at_pos
            .iter()
            .filter(|&obj| obj.name == "Stairs")
            .count()
            > 0;

        if !on_stairs {
            self.add_to_log("Can't go down, not standing on stairs.", Color::default());
            return false;
        }

        // clear the action queue, so enemies from the previous floor stop taking actions
        self.action_queue = BinaryHeap::new();

        // NOTE: code to generate next stage
        let cur_level = self.gamemap.level;
        self.generate_dungeon(DungeonConfig::default().set_level(cur_level + 1));
        self.add_to_log(
            "As you dive deeper into the dungeon, you find a moment to rest and recover.",
            Color::Magenta,
        );
        self.add_to_log("You feel stronger.", Color::Magenta);
        self.update_fov(VIEW_RADIUS);

        let player_fighter = self
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
}
