use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
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

impl Widget for CharWidget {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let tx = area.x + self.x;
        let ty = area.y + self.y;
        if tx < area.right() && ty < area.bottom() {
            buf.get_mut(tx, ty)
                .set_symbol(&self.ch.to_string())
                .set_style(Style::default().fg(Color::Yellow));
        }
    }
}

struct App {
    player_x: u16,
    player_y: u16,
}

impl App {
    fn new() -> Self {
        Self {
            player_x: 0,
            player_y: 0,
        }
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.player_x += 1;
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.player_x -= 1;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.player_y += 1;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.player_y -= 1;
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
        let ch = CharWidget {
            x: self.player_x,
            y: self.player_y,
            ch: '@',
        };
        frame.render_widget(ch, size);
    }
}
