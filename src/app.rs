use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use hecs::{Entity, World};
use ratatui::{DefaultTerminal, Frame, buffer::Buffer, widgets::Widget};

use crate::{
    components::{Object, Position, Renderable},
    entities,
    gamemap::{self, GameMap},
    procgen::generate_dungeon,
};

#[derive(Clone)]
pub struct CharWidget {
    position: Position,
    renderable: Renderable,
}

impl Widget for CharWidget {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let tx = area.x + self.position.x as u16;
        let ty = area.y + self.position.y as u16;
        if tx < area.right() && ty < area.bottom() {
            buf[(tx, ty)]
                .set_symbol(&self.renderable.glyph.to_string())
                .set_fg(self.renderable.fg)
                .set_bg(self.renderable.bg);
        }
    }
}

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

// get a clone of the position of an entity in the world
fn get_entity_position(entity: Entity, gamemap: &GameMap) -> Position {
    gamemap
        .world
        .get::<&Object>(entity)
        .unwrap()
        .position
        .clone()
}

// move an entity to (target_x, target_y)
fn move_action(entity: Entity, gamemap: &GameMap, (target_x, target_y): (u16, u16)) {
    if !gamemap.get_ref(target_x, target_y).walkable {
        return; // destination is blocked by a tile
    }
    if gamemap.get_blocking_entity_at_location(target_x, target_y) != None {
        // NOTE: this should get checked in bump_action
        return; // destination is blocked by an object
    }

    let mut obj = gamemap.world.get::<&mut Object>(entity).unwrap();
    let pos = &mut obj.position;
    pos.x = target_x;
    pos.y = target_y;
}

fn melee_action(entity: Entity, gamemap: &GameMap, (target_x, target_y): (u16, u16)) {
    let target = match gamemap.get_blocking_entity_at_location(target_x, target_y) {
        Some(x) => x,
        None => {
            return; // should never hit this case
        }
    };

    // TODO: implement actual melee attack code
    let source_obj = gamemap.world.get::<&Object>(entity).unwrap();
    let target_obj = gamemap.world.get::<&Object>(target).unwrap();
    println!("{} bumped into the {}...", source_obj.name, target_obj.name);
}

fn bump_action(entity: Entity, gamemap: &GameMap, direction: InputDirection) {
    // check that action target is in bounds
    let pos = get_entity_position(entity, gamemap);
    let deltas = direction_to_deltas(direction);
    let (dx, dy) = deltas;
    if !gamemap.in_bounds(pos.x as i16 + dx, pos.y as i16 + dy) {
        return; // destination is not in bounds
    }
    let (target_x, target_y) = ((pos.x as i16 + dx) as u16, (pos.y as i16 + dy) as u16);

    // decide which action to take
    match gamemap.get_blocking_entity_at_location(target_x, target_y) {
        Some(_) => {
            melee_action(entity, gamemap, (target_x, target_y));
        }
        None => {
            move_action(entity, gamemap, (target_x, target_y));
        }
    };
}

pub struct App {
    gamemap: GameMap,
    player: Entity,
}

impl App {
    pub fn new() -> Self {
        let mut world = World::new();
        let player = world.spawn(entities::player(0, 0));

        let max_rooms = 30;
        let room_min_size = 6;
        let room_max_size = 10;
        let max_monsters_per_room = 2;

        let dungeon_width = 80;
        let dungeon_height = 24;

        let mut gamemap = generate_dungeon(
            max_rooms,
            room_min_size,
            room_max_size,
            max_monsters_per_room,
            dungeon_width,
            dungeon_height,
            world,
            player,
        );

        gamemap.update_fov(player, 8);

        Self { gamemap, player }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => {
                        break Ok(());
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        bump_action(self.player, &mut self.gamemap, InputDirection::Right);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        bump_action(self.player, &mut self.gamemap, InputDirection::Left);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        bump_action(self.player, &mut self.gamemap, InputDirection::Down);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        bump_action(self.player, &mut self.gamemap, InputDirection::Up);
                    }
                    KeyCode::Char('u') => {
                        bump_action(self.player, &mut self.gamemap, InputDirection::UpRight);
                    }
                    KeyCode::Char('y') => {
                        bump_action(self.player, &mut self.gamemap, InputDirection::UpLeft);
                    }
                    KeyCode::Char('n') => {
                        bump_action(self.player, &mut self.gamemap, InputDirection::DownRight);
                    }
                    KeyCode::Char('b') => {
                        bump_action(self.player, &mut self.gamemap, InputDirection::DownLeft);
                    }
                    _ => {}
                }

                let view_radius = 8;
                self.gamemap.update_fov(self.player, view_radius);
            }
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        self.render_map(frame);
        self.render_entities(frame);
    }

    // render tiles in gamemap
    fn render_map(&self, frame: &mut Frame) {
        for x in 0..self.gamemap.width {
            for y in 0..self.gamemap.height {
                let tile = self.gamemap.get_ref(x, y);
                let ch = CharWidget {
                    position: Position { x, y },
                    renderable: {
                        if self.gamemap.is_visible(x, y) {
                            tile.light.clone()
                        } else if self.gamemap.is_explored(x, y) {
                            tile.dark.clone()
                        } else {
                            gamemap::shroud_renderable()
                        }
                    },
                };
                frame.render_widget(ch, frame.area());
            }
        }
    }

    // render entities in the world
    fn render_entities(&self, frame: &mut Frame) {
        let size = frame.area();
        for (_entity, obj) in self.gamemap.world.query::<&Object>().iter() {
            let position = &obj.position;
            let renderable = &obj.renderable;

            // render only visible entities
            if !self.gamemap.is_visible(position.x, position.y) {
                continue;
            }

            let ch = CharWidget {
                position: position.clone(),
                renderable: renderable.clone(),
            };
            frame.render_widget(ch, size);
        }
    }
}
