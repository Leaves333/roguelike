use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use hecs::World;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

struct CharWidget {
    x: u16,
    y: u16,
    ch: char,
}

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Renderable {
    pub glyph: char,
    pub color: Color,
}

pub struct Player {}

impl Widget for CharWidget {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let tx = area.x + self.x;
        let ty = area.y + self.y;
        if tx < area.right() && ty < area.bottom() {
            buf[(tx, ty)]
                .set_symbol(&self.ch.to_string())
                .set_style(Style::default().fg(Color::Yellow));
        }
    }
}

struct App {
    // player_x: u16,
    // player_y: u16,
    world: World,
}

enum InputDirection {
    Up,
    Down,
    Left,
    Right,
}

fn player_input_system(world: &mut World, input: InputDirection) {
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
    fn new() -> Self {
        Self {
            world: {
                let mut x = World::new();
                x.spawn((
                    Player {},
                    Position { x: 0, y: 0 },
                    Renderable {
                        glyph: '@',
                        color: Color::White,
                    },
                ));
                x.spawn((
                    Position { x: 1, y: 3 },
                    Renderable {
                        glyph: 'h',
                        color: Color::White,
                    },
                ));
                x
            },
        }
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Right | KeyCode::Char('l') => {
                        player_input_system(&mut self.world, InputDirection::Right);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        player_input_system(&mut self.world, InputDirection::Left);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        player_input_system(&mut self.world, InputDirection::Down);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        player_input_system(&mut self.world, InputDirection::Up);
                    }
                    KeyCode::Esc => {
                        break Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        let size = frame.area();

        // Optional background box
        let block = Block::default().title("Demo").borders(Borders::ALL);
        frame.render_widget(block, size);

        // Draw the character at (x, y)
        for (_entity, (pos, renderable)) in self.world.query::<(&Position, &Renderable)>().iter() {
            let ch = CharWidget {
                x: pos.x as u16,
                y: pos.y as u16,
                ch: renderable.glyph,
            };
            frame.render_widget(ch, size);
        }
    }
}
