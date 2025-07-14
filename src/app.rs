use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use hecs::World;
use ratatui::{DefaultTerminal, Frame, buffer::Buffer, style::Color, widgets::Widget};

use crate::{
    gamemap::{GameMap, Tile, TileType},
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
        }
    }
}

pub struct App {
    world: World,
    gamemap: GameMap,
}

impl App {
    pub fn new() -> Self {
        Self {
            world: {
                let mut x = World::new();
                x.spawn((
                    Player {},
                    Position { x: 25, y: 7 },
                    Renderable {
                        glyph: '@',
                        fg: Color::default(), // NOTE: default color is white text color
                        bg: Color::Reset,
                    },
                ));
                x.spawn((
                    Position { x: 1, y: 3 },
                    Renderable {
                        glyph: 'h',
                        fg: Color::Yellow,
                        bg: Color::Reset,
                    },
                ));
                x
            },
            gamemap: {
                // let mut gamemap = GameMap::new(80, 24);
                // *gamemap.get_mut(3, 3) = Tile::from_type(TileType::Wall);
                // gamemap
                generate_dungeon(80, 24)
            },
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
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
                    KeyCode::Esc => {
                        break Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let size = frame.area();

        // render tiles
        for x in 0..self.gamemap.width {
            for y in 0..self.gamemap.height {
                let tile = self.gamemap.get_ref(x, y);
                let ch = CharWidget {
                    position: Position {
                        x: x.into(),
                        y: y.into(),
                    },
                    renderable: tile.dark.clone(),
                };
                frame.render_widget(ch, size);
            }
        }

        // draw the character at (x, y)
        for (_entity, (position, renderable)) in
            self.world.query::<(&Position, &Renderable)>().iter()
        {
            let ch = CharWidget {
                position: position.clone(),
                renderable: renderable.clone(),
            };
            frame.render_widget(ch, size);
        }
    }
}
