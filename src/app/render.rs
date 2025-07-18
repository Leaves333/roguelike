use ratatui::{
    Frame,
    buffer::Buffer,
    layout,
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

use super::App;
use crate::{
    components::{Object, Position, Renderable},
    gamemap,
};

#[derive(Clone)]
pub struct CharWidget {
    position: Position,
    renderable: Renderable,
}

impl Widget for CharWidget {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        // add and subtract 1 to account for borders
        let tx = area.x + self.position.x + 1 as u16;
        let ty = area.y + self.position.y + 1 as u16;
        if tx < area.right() - 1 && ty < area.bottom() - 1 {
            buf[(tx, ty)]
                .set_symbol(&self.renderable.glyph.to_string())
                .set_fg(self.renderable.fg)
                .set_bg(self.renderable.bg);
        }
    }
}

impl App {
    pub fn render(&self, frame: &mut Frame) {
        let layout = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(vec![
                layout::Constraint::Percentage(70),
                layout::Constraint::Min(5),
            ])
            .split(frame.area());
        self.render_map(frame, layout[0]);
        self.render_entities(frame, layout[0]);
        self.render_log(frame, layout[1]);
    }

    // render tiles in gamemap
    fn render_map(&self, frame: &mut Frame, area: layout::Rect) {
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
                frame.render_widget(ch, area);
            }
        }
    }

    // render entities in the world
    fn render_entities(&self, frame: &mut Frame, area: layout::Rect) {
        let block = Block::default().title("world").borders(Borders::ALL);
        frame.render_widget(block, area);

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
            frame.render_widget(ch, area);
        }
    }

    fn render_log(&self, frame: &mut Frame, area: layout::Rect) {
        let mut lines: Vec<Line> = self.log.iter().map(|s| Line::from(s.as_str())).collect();
        let display_idx = lines.len().saturating_sub(area.height as usize - 2);
        let bottom_lines = lines.split_off(display_idx);

        let paragraph =
            Paragraph::new(bottom_lines).block(Block::default().title("log").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }
}
