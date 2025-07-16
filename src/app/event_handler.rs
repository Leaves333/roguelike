use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use hecs::Entity;
use ratatui::DefaultTerminal;

use crate::components::{Object, Position};

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
                let view_radius = 8;
                self.gamemap.update_fov(self.player, view_radius);
            }
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
            // NOTE: this should get checked in bump_action
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
