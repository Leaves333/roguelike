use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use hecs::Entity;
use ratatui::DefaultTerminal;

use crate::components::{Fighter, MeleeAI, Object, Position};
use crate::gamemap::coords_to_idx;
use crate::pathfinding::Pathfinder;

use super::App;

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

impl App {
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()? {
                // player takes an action...
                self.log.push(String::from("### new turn"));
                match key.code {
                    KeyCode::Esc => {
                        break Ok(());
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.bump_action(self.player, InputDirection::Right);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.bump_action(self.player, InputDirection::Left);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.bump_action(self.player, InputDirection::Down);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.bump_action(self.player, InputDirection::Up);
                    }
                    KeyCode::Char('u') => {
                        self.bump_action(self.player, InputDirection::UpRight);
                    }
                    KeyCode::Char('y') => {
                        self.bump_action(self.player, InputDirection::UpLeft);
                    }
                    KeyCode::Char('n') => {
                        self.bump_action(self.player, InputDirection::DownRight);
                    }
                    KeyCode::Char('b') => {
                        self.bump_action(self.player, InputDirection::DownLeft);
                    }
                    KeyCode::Char('5') | KeyCode::Char('.') => {
                        // wait action
                    }
                    _ => {}
                }

                // monsters act...
                self.handle_monster_turns();

                // update fov
                let view_radius = 8;
                self.gamemap.update_fov(self.player, view_radius);
                self.log.push(String::from(""));
            }
        }
    }

    // makes all monsters take a turn...
    fn handle_monster_turns(&mut self) {
        self.log.push(String::from("handling monster turns!"));
        self.handle_melee_ai();
    }

    // moves all entities with melee ai
    fn handle_melee_ai(&mut self) {
        let mut queued_move_actions = Vec::new();
        let mut queued_melee_actions = Vec::new();

        for (monster_ent, (monster_obj, fighter, ai)) in self
            .gamemap
            .world
            .query::<(&Object, &Fighter, &MeleeAI)>()
            .iter()
        {
            let monster_pos = &monster_obj.position;
            let player_pos = self.get_entity_position(self.player);
            let out_of_range = monster_pos.x.abs_diff(player_pos.x) > 8
                || monster_pos.y.abs_diff(player_pos.y) > 8;

            if out_of_range {
                continue;
            }

            // self.log.push(format!(
            //     "monster {} is in range of the player!",
            //     monster_obj.name
            // ));

            // NOTE: rework los algorithm later, for now assume it is symmetric
            if !self.gamemap.is_visible(monster_pos.x, monster_pos.y) {
                continue;
            }

            self.log
                .push(format!("monster {} can see the player!!", monster_obj.name));

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
                (monster_pos.x, monster_pos.y),
                self.gamemap.width,
                self.gamemap.height,
                2,
                3,
            );

            let path = pathfinder.path_to((player_pos.x, player_pos.y));
            if path.len() == 0 {
                self.log
                    .push(format!("{} just sits and waits.", monster_obj.name));
                continue;
            } else if path.len() == 1 {
                queued_melee_actions.push((monster_ent, *path.first().unwrap()));
            } else {
                self.log
                    .push(format!("{} moves towards the player!", monster_obj.name));
                queued_move_actions.push((monster_ent, *path.first().unwrap()));
            }
        }

        for (entity, dest) in queued_move_actions {
            self.move_action(entity, dest);
        }
        for (entity, dest) in queued_melee_actions {
            self.melee_action(entity, dest);
        }
    }

    // get a clone of the position of an entity in the world
    fn get_entity_position(&self, entity: Entity) -> Position {
        self.gamemap
            .world
            .get::<&Object>(entity)
            .unwrap()
            .position
            .clone()
    }

    // move an entity to (target_x, target_y)
    fn move_action(&self, entity: Entity, (target_x, target_y): (u16, u16)) {
        if !self.gamemap.get_ref(target_x, target_y).walkable {
            return; // destination is blocked by a tile
        }
        if self
            .gamemap
            .get_blocking_entity_at_location(target_x, target_y)
            != None
        {
            return; // destination is blocked by an object
        }

        let mut obj = self.gamemap.world.get::<&mut Object>(entity).unwrap();
        let pos = &mut obj.position;
        pos.x = target_x;
        pos.y = target_y;
    }

    fn melee_action(&mut self, entity: Entity, (target_x, target_y): (u16, u16)) {
        let target = match self
            .gamemap
            .get_blocking_entity_at_location(target_x, target_y)
        {
            Some(x) => x,
            None => {
                return; // should never hit this case
            }
        };

        // TODO: implement actual melee attack code
        let source_obj = self.gamemap.world.get::<&Object>(entity).unwrap();
        let source_fighter = self.gamemap.world.get::<&Fighter>(entity).unwrap();
        let target_obj = self.gamemap.world.get::<&Object>(target).unwrap();
        let mut target_fighter = self.gamemap.world.get::<&mut Fighter>(target).unwrap();

        let damage = (source_fighter.power - target_fighter.defense).max(0) as u16;
        let attack_desc = format!("{} attacks {}", source_obj.name, target_obj.name);

        if damage > 0 {
            let target_hp = target_fighter.get_hp().saturating_sub(damage);
            target_fighter.set_hp(target_hp);
            self.log
                .push(format!("{} for {} damage.", attack_desc, damage));
        } else {
            self.log
                .push(format!("{} but does no damage.", attack_desc));
        }
    }

    fn bump_action(&mut self, entity: Entity, direction: InputDirection) {
        // check that action target is in bounds
        let pos = self.get_entity_position(entity);
        let deltas = direction_to_deltas(direction);
        let (dx, dy) = deltas;
        if !self.gamemap.in_bounds(pos.x as i16 + dx, pos.y as i16 + dy) {
            return; // destination is not in bounds
        }
        let (target_x, target_y) = ((pos.x as i16 + dx) as u16, (pos.y as i16 + dy) as u16);

        // decide which action to take
        match self
            .gamemap
            .get_blocking_entity_at_location(target_x, target_y)
        {
            Some(_) => {
                self.melee_action(entity, (target_x, target_y));
            }
            None => {
                self.move_action(entity, (target_x, target_y));
            }
        };
    }
}
