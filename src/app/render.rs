use std::fmt::format;

use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout,
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Borders, LineGauge, Paragraph, Widget},
};

use super::{App, PLAYER};
use crate::{
    components::{Position, Renderable},
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
        let horizontal_split = layout::Layout::default()
            .direction(layout::Direction::Horizontal)
            .constraints(vec![
                layout::Constraint::Min(15),
                layout::Constraint::Percentage(70),
            ])
            .split(frame.area());

        let ui_layout = horizontal_split[0];
        let world_layout = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(vec![
                layout::Constraint::Percentage(70),
                layout::Constraint::Min(5),
            ])
            .split(horizontal_split[1]);

        self.render_map(frame, world_layout[0]);
        self.render_entities(frame, world_layout[0]);
        self.render_log(frame, world_layout[1]);
        self.render_status(frame, ui_layout);
    }

    /// render tiles in gamemap
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

    /// render all objects in the gamemap to screen
    fn render_entities(&self, frame: &mut Frame, area: layout::Rect) {
        let block = Block::default().title("world").borders(Borders::ALL);
        frame.render_widget(block, area);

        let mut to_draw = self.gamemap.objects.clone();
        to_draw.sort_by(|a, b| a.blocks_movement.cmp(&b.blocks_movement));

        for obj in to_draw {
            let position = &obj.pos;
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

    /// renders the text in the log
    fn render_log(&self, frame: &mut Frame, area: layout::Rect) {
        let mut lines: Vec<Line> = self.log.iter().map(|s| Line::from(s.as_str())).collect();
        let display_idx = lines.len().saturating_sub(area.height as usize - 2);
        let bottom_lines = lines.split_off(display_idx);

        let paragraph =
            Paragraph::new(bottom_lines).block(Block::default().title("log").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }

    /// renders inventory ui on the left side of the screen
    fn render_status(&self, frame: &mut Frame, area: layout::Rect) {
        let player = &self.gamemap.objects[PLAYER];
        let fighter = &player.fighter.as_ref().unwrap();
        let ratio = fighter.hp as f64 / fighter.max_hp as f64;

        let gauge = LineGauge::default()
            .block(Block::bordered().title("status"))
            .filled_style(Style::new().on_blue())
            .line_set(symbols::line::NORMAL)
            .label(format!("HP: {}/{}", fighter.hp, fighter.max_hp))
            .ratio(ratio);

        frame.render_widget(gauge, area);
    }
}
