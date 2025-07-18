use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use hecs::Entity;
use ratatui::DefaultTerminal;

use crate::components::{Fighter, MeleeAI, Object, Position};
use crate::gamemap::coords_to_idx;
use crate::los;
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
                    _ => {}
                }

                // monsters act...
                self.handle_monster_turns();

                // update fov
                let view_radius = 8;
                self.gamemap.update_fov(self.player, view_radius);
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

            self.log.push(format!(
                "monster {} is in range of the player!",
                monster_obj.name
            ));

            // path to the player and check if it has line of sight
            // let has_los = los::bresenham(
            //     (player_pos.x as i32, player_pos.y as i32),
            //     (monster_pos.x as i32, monster_pos.y as i32),
            // )
            // .iter()
            // .map(|(x, y)| (*x as u16, *y as u16))
            // .fold(true, |b, (x, y)| {
            //     b && self.gamemap.in_bounds(x as i16, y as i16)
            //         && self.gamemap.get_ref(x, y).transparent
            // });
            //
            // if !has_los {
            //     continue;
            // }

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
            } else {
                self.log
                    .push(format!("{} moves towards the player!", monster_obj.name));
                queued_move_actions.push((monster_ent, *path.first().unwrap()));
                // self.move_action(monster_ent, *path.first().unwrap());
            }
        }

        for (entity, dest) in queued_move_actions {
            self.move_action(entity, dest);
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
        let target_obj = self.gamemap.world.get::<&Object>(target).unwrap();

        self.log.push(format!(
            "{} bumped into the {}",
            source_obj.name, target_obj.name
        ));
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
