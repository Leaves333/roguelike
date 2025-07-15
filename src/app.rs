use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use hecs::{Entity, World};
use ratatui::{DefaultTerminal, Frame, buffer::Buffer, style::Color, widgets::Widget};

use crate::{
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

#[derive(Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
    pub bg: Color,
}

pub struct Player {}

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

fn move_entity(gamemap: &GameMap, pos: &mut Position, dx: i16, dy: i16) {
    if gamemap.in_bounds(pos.x as i16 + dx, pos.y as i16 + dy) {
        let new_x = (pos.x as i16 + dx) as u16;
        let new_y = (pos.y as i16 + dy) as u16;

        if !gamemap.get_ref(new_x, new_y).walkable {
            return;
        }

        pos.x = new_x;
        pos.y = new_y;
    }
}

fn move_player(world: &mut World, gamemap: &GameMap, input: InputDirection) {
    // query for the player
    for (_entity, (pos, _player)) in world.query_mut::<(&mut Position, &Player)>() {
        match input {
            InputDirection::Up => move_entity(gamemap, pos, 0, -1),
            InputDirection::Down => move_entity(gamemap, pos, 0, 1),
            InputDirection::Left => move_entity(gamemap, pos, -1, 0),
            InputDirection::Right => move_entity(gamemap, pos, 1, 0),
            InputDirection::UpLeft => move_entity(gamemap, pos, -1, -1),
            InputDirection::UpRight => move_entity(gamemap, pos, 1, -1),
            InputDirection::DownLeft => move_entity(gamemap, pos, -1, 1),
            InputDirection::DownRight => move_entity(gamemap, pos, 1, 1),
        }
    }
}

pub struct App {
    world: World,
    gamemap: GameMap,
    player: Entity,
}

impl App {
    pub fn new() -> Self {
        let mut world = World::new();
        let player = world.spawn((
            Player {},
            Position { x: 0, y: 0 },
            Renderable {
                glyph: '@',
                fg: Color::default(), // NOTE: default color is white text color
                bg: Color::Reset,
            },
        ));

        // dummy npc
        world.spawn((
            Position { x: 1, y: 3 },
            Renderable {
                glyph: 'h',
                fg: Color::Yellow,
                bg: Color::Reset,
            },
        ));

        let max_rooms = 30;
        let room_min_size = 6;
        let room_max_size = 10;

        let dungeon_width = 80;
        let dungeon_height = 24;

        let mut position = world.get::<&mut Position>(player).unwrap();
        let mut gamemap = generate_dungeon(
            max_rooms,
            room_min_size,
            room_max_size,
            dungeon_width,
            dungeon_height,
            &mut *position,
        );
        gamemap.update_fov(&position, 8);

        drop(position);

        Self {
            world,
            gamemap,
            player,
        }
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
                        move_player(&mut self.world, &self.gamemap, InputDirection::Right);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        move_player(&mut self.world, &self.gamemap, InputDirection::Left);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        move_player(&mut self.world, &self.gamemap, InputDirection::Down);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        move_player(&mut self.world, &self.gamemap, InputDirection::Up);
                    }
                    KeyCode::Char('u') => {
                        move_player(&mut self.world, &self.gamemap, InputDirection::UpRight);
                    }
                    KeyCode::Char('y') => {
                        move_player(&mut self.world, &self.gamemap, InputDirection::UpLeft);
                    }
                    KeyCode::Char('n') => {
                        move_player(&mut self.world, &self.gamemap, InputDirection::DownRight);
                    }
                    KeyCode::Char('b') => {
                        move_player(&mut self.world, &self.gamemap, InputDirection::DownLeft);
                    }
                    _ => {}
                }

                let position = self.world.get::<&Position>(self.player).unwrap();
                let view_radius = 8;
                self.gamemap.update_fov(&position, view_radius);
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
                    position: Position { x: x, y: y },
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
        for (_entity, (position, renderable)) in
            self.world.query::<(&Position, &Renderable)>().iter()
        {
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
