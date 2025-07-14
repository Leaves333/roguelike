use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use hecs::World;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

pub struct App {
    world: World,
}

#[derive(Clone)]
pub struct CharWidget {
    position: Position,
    renderable: Renderable,
}

#[derive(Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub struct Renderable {
    pub glyph: char,
    pub color: Color,
}

pub struct Player {}

impl Widget for CharWidget {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let tx = area.x + self.position.x as u16;
        let ty = area.y + self.position.y as u16;
        if tx < area.right() && ty < area.bottom() {
            buf[(tx, ty)]
                .set_symbol(&self.renderable.glyph.to_string())
                .set_style(Style::default().fg(self.renderable.color));
        }
    }
}

enum InputDirection {
    Up,
    Down,
    Left,
    Right,
}

fn move_player(world: &mut World, input: InputDirection) {
    // query for the player
    for (_entity, (pos, _player)) in world.query_mut::<(&mut Position, &Player)>() {
        match input {
            InputDirection::Up => pos.y -= 1,
            InputDirection::Down => pos.y += 1,
            InputDirection::Left => pos.x -= 1,
            InputDirection::Right => pos.x += 1,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            world: {
                let mut x = World::new();
                x.spawn((
                    Player {},
                    Position { x: 0, y: 0 },
                    Renderable {
                        glyph: '@',
                        color: Color::DarkGray,
                    },
                ));
                x.spawn((
                    Position { x: 1, y: 3 },
                    Renderable {
                        glyph: 'h',
                        color: Color::Black,
                    },
                ));
                x.spawn((
                    Position { x: 1, y: 4 },
                    Renderable {
                        glyph: '@',
                        color: Color::DarkGray,
                    },
                ));
                x.spawn((
                    Position { x: 1, y: 5 },
                    Renderable {
                        glyph: '@',
                        color: Color::Gray,
                    },
                ));
                x.spawn((
                    Position { x: 1, y: 6 },
                    Renderable {
                        glyph: '@',
                        color: Color::White,
                    },
                ));
                x.spawn((
                    Position { x: 1, y: 7 },
                    Renderable {
                        glyph: '@',
                        color: Color::LightMagenta,
                    },
                ));
                x
            },
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Right | KeyCode::Char('l') => {
                        move_player(&mut self.world, InputDirection::Right);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        move_player(&mut self.world, InputDirection::Left);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        move_player(&mut self.world, InputDirection::Down);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        move_player(&mut self.world, InputDirection::Up);
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

        // Optional background box
        let block = Block::default().title("Demo").borders(Borders::ALL);
        frame.render_widget(block, size);

        // Draw the character at (x, y)
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
