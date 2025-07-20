use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;

use crate::components::AIType;
use crate::gamemap::coords_to_idx;
use crate::pathfinding::Pathfinder;

use super::App;

const PLAYER: usize = 0;

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
                // player takes an action...
                self.log.push(String::from("### new turn"));
                match key.code {
                    KeyCode::Esc => {
                        break Ok(());
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.bump_action(PLAYER, InputDirection::Right);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.bump_action(PLAYER, InputDirection::Left);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.bump_action(PLAYER, InputDirection::Down);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.bump_action(PLAYER, InputDirection::Up);
                    }
                    KeyCode::Char('u') => {
                        self.bump_action(PLAYER, InputDirection::UpRight);
                    }
                    KeyCode::Char('y') => {
                        self.bump_action(PLAYER, InputDirection::UpLeft);
                    }
                    KeyCode::Char('n') => {
                        self.bump_action(PLAYER, InputDirection::DownRight);
                    }
                    KeyCode::Char('b') => {
                        self.bump_action(PLAYER, InputDirection::DownLeft);
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
                self.gamemap.update_fov(view_radius);
                self.log.push(String::from(""));
            }
        }
    }

    /// makes all the monsters take a turn
    fn handle_monster_turns(&mut self) {
        self.log.push(String::from("handling monster turns!"));
        for i in 0..self.gamemap.objects.len() {
            let obj = &self.gamemap.objects[i];
            if !obj.alive {
                continue;
            }

            if let Some(ai_type) = &obj.ai {
                match ai_type {
                    AIType::Melee => {
                        self.handle_melee_ai(i);
                    }
                }
            }
        }
    }

    /// makes a monster act according to melee ai
    fn handle_melee_ai(&mut self, id: usize) {
        let monster = &self.gamemap.objects[id];
        let player = &self.gamemap.objects[PLAYER];
        let out_of_range =
            monster.pos.x.abs_diff(player.pos.x) > 8 || monster.pos.y.abs_diff(player.pos.y) > 8;

        if out_of_range {
            return;
        }

        // NOTE: rework los algorithm later, for now assume it is symmetric
        if !self.gamemap.is_visible(monster.pos.x, monster.pos.y) {
            return;
        }

        self.log
            .push(format!("monster {} can see the player!!", monster.name));

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

        if let Some(_) = self.gamemap.get_blocking_object_id(target_x, target_y) {
            return; // destination is blocked by an object
        }

        let pos = &mut self.gamemap.objects[id].pos;
        pos.x = target_x;
        pos.y = target_y;
    }

    fn melee_action(&mut self, attacker_id: usize, (target_x, target_y): (u16, u16)) {
        // check that there is an object to attack
        let target_id = match self.gamemap.get_blocking_object_id(target_x, target_y) {
            Some(x) => x,
            None => {
                return; // should never hit this case
            }
        };

        // TODO: implement actual melee attack code
        let (attacker, target) = mut_two(attacker_id, target_id, &mut self.gamemap.objects);
        let attacker_fighter = &attacker.fighter.as_ref().unwrap();
        let target_fighter = &mut target.fighter.as_mut().unwrap();

        let damage = (attacker_fighter.power - target_fighter.defense).max(0) as u16;
        let attack_desc = format!("{} attacks {}", attacker.name, target.name);

        if damage > 0 {
            target.take_damage(damage);
            self.log
                .push(format!("{} for {} damage.", attack_desc, damage));
        } else {
            self.log
                .push(format!("{} but does no damage.", attack_desc));
        }
    }

    fn bump_action(&mut self, id: usize, direction: InputDirection) {
        // check that action target is in bounds
        let pos = &self.gamemap.objects[id].pos;
        let deltas = direction_to_deltas(direction);
        let (dx, dy) = deltas;
        if !self.gamemap.in_bounds(pos.x as i16 + dx, pos.y as i16 + dy) {
            return; // destination is not in bounds
        }
        let (target_x, target_y) = ((pos.x as i16 + dx) as u16, (pos.y as i16 + dy) as u16);

        // decide which action to take
        match self.gamemap.get_blocking_object_id(target_x, target_y) {
            Some(_) => {
                self.melee_action(id, (target_x, target_y));
            }
            None => {
                self.move_action(id, (target_x, target_y));
            }
        };
    }
}
