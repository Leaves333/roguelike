use core::panic;
use std::collections::HashSet;

use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::style::Color;

use crate::components::{
    AIType, Item, MELEE_FORGET_TIME, MeleeAIData, Object, Position, RenderStatus, SLOT_ORDERING,
};
use crate::engine::{UseResult, defense, power, take_damage};
use crate::gamemap::coords_to_idx;
use crate::los;
use crate::pathfinding::Pathfinder;

use super::procgen::DungeonConfig;
use super::{App, GameScreen, INVENTORY_SIZE, PLAYER, VIEW_RADIUS};

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
                        self.update_fov(VIEW_RADIUS);

                        // increment the time
                        self.time += 100;
                    }
                    PlayerAction::DidntTakeTurn => {
                        // nothing happens
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

        // movement related controls
        match self.game_screen {
            GameScreen::Main => match key.code {
                // movement keys during the main screen
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
                _ => {}
            },
            GameScreen::Examine { ref mut cursor }
            | GameScreen::Targeting { ref mut cursor, .. } => match key.code {
                // move cursor around during examine and targeting modes
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

                KeyCode::Char('u') => {
                    cursor.x = (cursor.x + 1).min(self.gamemap.width - 1);
                    cursor.y = cursor.y.saturating_sub(1);
                }
                KeyCode::Char('y') => {
                    cursor.x = cursor.x.saturating_sub(1);
                    cursor.y = cursor.y.saturating_sub(1);
                }
                KeyCode::Char('n') => {
                    cursor.x = (cursor.x + 1).min(self.gamemap.width - 1);
                    cursor.y = (cursor.y + 1).min(self.gamemap.height - 1);
                }
                KeyCode::Char('b') => {
                    cursor.x = cursor.x.saturating_sub(1);
                    cursor.y = (cursor.y + 1).min(self.gamemap.height - 1);
                }
                _ => {}
            },
            _ => {}
        };

        // keybinds specific to certain gamescreens
        match self.game_screen {
            // main menu controls
            GameScreen::Menu => {
                match key.code {
                    KeyCode::Char('n') => {
                        // start new game
                        self.new_game();
                        self.switch_to_main_screen();
                    }
                    KeyCode::Char('l') => {
                        // loads an existing game from a save file
                        let _ = self.load_game();
                        self.switch_to_main_screen();
                    }
                    KeyCode::Char('q') => {
                        // quit the game
                        return PlayerAction::Exit;
                    }
                    _ => {}
                }
            }
            GameScreen::Main => {
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
                                self.drop_item_from_inventory(index);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

                match key.code {
                    // use item from inventory
                    KeyCode::Char(c @ '1'..='9') | KeyCode::Char(c @ '0') => {
                        let index = match c {
                            '1'..='9' => c as usize - '1' as usize,
                            '0' => 9,
                            _ => unreachable!(),
                        };

                        if self.inventory.len() > index {
                            let item = self.get_item_in_inventory(index).clone();

                            if item.needs_targeting() {
                                // item needs targeting, switch to targeting mode
                                item.on_targeting(self, index);
                                return PlayerAction::DidntTakeTurn;
                            } else {
                                // item can be used directly
                                let use_result = self.use_item(index, None);
                                return match use_result {
                                    UseResult::UsedUp => PlayerAction::TookTurn,
                                    UseResult::Equipped => PlayerAction::TookTurn,
                                    UseResult::Cancelled => PlayerAction::DidntTakeTurn,
                                };
                            }
                        }
                    }

                    // unequip item from equipment
                    KeyCode::Char(c @ 'A'..='C') => {
                        let index = c as usize - 'A' as usize;
                        match self.equipment[index] {
                            Some(id) => {
                                // check we have enough space in inventory to unequip the item
                                if self.inventory.len() >= INVENTORY_SIZE {
                                    self.log
                                        .add("Not enough space in inventory.", Color::default());
                                    return PlayerAction::DidntTakeTurn;
                                }

                                // unequip and move to inventory
                                self.inventory.push(id);
                                self.equipment[index] = None;
                            }
                            None => {
                                self.log.add(
                                    format!("No item equipped on {}.", SLOT_ORDERING[index]),
                                    Color::default(),
                                );
                            }
                        }
                    }

                    // pick up the first item at location
                    KeyCode::Char('g') => {
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

                    // can only go to examine mode from main game screen
                    KeyCode::Char('x') => {
                        self.toggle_examine_mode();
                        return PlayerAction::DidntTakeTurn;
                    }

                    // go down stairs if stairs exist
                    KeyCode::Char('>') => {
                        let _ = self.go_down_stairs();
                        self.switch_to_main_screen();
                        return PlayerAction::DidntTakeTurn;
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
            GameScreen::Examine { .. } => match key.code {
                // exit examine mode
                KeyCode::Char('x') => {
                    self.toggle_examine_mode();
                    return PlayerAction::DidntTakeTurn;
                }
                _ => {}
            },
            GameScreen::Targeting {
                ref cursor,
                inventory_idx,
                ..
            } => match key.code {
                KeyCode::Enter => {
                    // use the item and exit targeting mode
                    let use_result = self.use_item(inventory_idx, Some(cursor.clone()));
                    self.game_screen = GameScreen::Main;

                    return match use_result {
                        UseResult::UsedUp => PlayerAction::TookTurn,
                        UseResult::Equipped => PlayerAction::TookTurn,
                        UseResult::Cancelled => PlayerAction::DidntTakeTurn,
                    };
                }
                _ => {}
            },
        };

        // if no keybinds were matched
        return PlayerAction::DidntTakeTurn;
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
                    AIType::Melee(_) => {
                        self.handle_melee_ai(id.clone());
                    }
                    AIType::Ranged => {
                        todo!()
                    }
                }
            }
        }
    }

    /// makes a monster act according to melee ai
    /// assumes that said monster has an MeleeAI component
    fn handle_melee_ai(&mut self, id: usize) {
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

        let target = match ai_data.target {
            Some(id) => id,
            None => {
                // do nothing if no current target
                return;
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
        } else if path.len() == 1 {
            self.melee_action(id, *path.first().unwrap());
        } else {
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
            take_damage(&mut self.objects, &mut self.log, target_id, damage);
            self.log.add(
                format!("{} for {} damage.", attack_desc, damage),
                Color::default(),
            );
        } else {
            self.log.add(
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
        if self.inventory.len() >= INVENTORY_SIZE {
            self.log
                .add(format!("Cannot hold that many items."), Color::default());
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

                    self.log
                        .add(format!("Picked up {}.", item_obj.name), Color::default());
                }
                None => {
                    panic!("invalid object id passed to pick_item_up()!")
                }
            }
        }
    }

    /// drop an item back onto the ground
    fn drop_item_from_inventory(&mut self, inventory_idx: usize) {
        if inventory_idx > self.inventory.len() {
            self.log.add("No item to drop.", Color::default());
            return;
        }

        // reshow the item on the map, and set its position to the player's position
        let player_pos = self.objects.get(&PLAYER).unwrap().pos.clone();
        let item_id = self.inventory[inventory_idx];
        let item_obj = self.objects.get_mut(&item_id).unwrap();

        self.gamemap.object_ids.push(item_id);
        item_obj.pos = player_pos;
        item_obj.render_status = RenderStatus::ShowInFOV;

        self.inventory.remove(inventory_idx);
    }

    /// returns the item for a given index in the inventory
    fn get_item_in_inventory(&self, inventory_idx: usize) -> &Item {
        let item_id = self.inventory[inventory_idx];
        match &self.objects.get(&item_id).unwrap().item {
            Some(x) => x,
            None => {
                panic!(
                    "get_item_in_inventory() called, but object does not have an item component!"
                )
            }
        }
    }

    /// returns the object for a given index in the inventory
    fn get_object_in_inventory(&self, inventory_idx: usize) -> &Object {
        let item_id = self.inventory[inventory_idx];
        match self.objects.get(&item_id) {
            Some(x) => x,
            None => {
                panic!(
                    "get_object_in_inventory() called, but could not find an object with that id!"
                )
            }
        }
    }

    /// uses an item from the specified index in the inventory
    fn use_item(&mut self, inventory_idx: usize, target: Option<Position>) -> UseResult {
        let item = self.get_item_in_inventory(inventory_idx).clone();
        let use_result = item.on_use(self, target);

        match use_result {
            UseResult::UsedUp => {
                // delete item after being used
                self.inventory.remove(inventory_idx);
            }
            UseResult::Cancelled => {
                // item wasn't used, don't delete it
            }
            UseResult::Equipped => {
                // try to equip item by moving it from the inventory to the equipment slot

                // get the index that this item is supposed to be equipped in
                let obj = self.get_object_in_inventory(inventory_idx);
                let equip = obj.equipment.as_ref().unwrap();
                let equip_idx = equip.slot as usize;

                // check if the slot is empty or not
                if self.equipment[equip_idx].is_some() {
                    self.log.add(
                        format!("Already have an item equipped on your {}!", equip.slot),
                        Color::default(),
                    );
                    return UseResult::Cancelled;
                }

                // if equipment slot isn't empty, equip it
                self.equipment[equip_idx] = Some(self.inventory[inventory_idx]);

                // remove equipped item from inventory
                self.inventory.remove(inventory_idx);
            }
        };

        use_result
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
            self.log
                .add("Can't go down, not standing on stairs.", Color::default());
            return false;
        }

        // NOTE: code to generate next stage
        let cur_level = self.gamemap.level;
        self.generate_dungeon(DungeonConfig::default().set_level(cur_level + 1));
        self.log.add(
            "As you dive deeper into the dungeon, you find a moment to rest and recover.",
            Color::Magenta,
        );
        self.log.add("You feel stronger.", Color::Magenta);
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
