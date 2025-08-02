use ratatui::{
    Frame,
    buffer::Buffer,
    layout,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use super::{App, PLAYER};
use crate::{
    components::{Position, RenderStatus, Renderable},
    gamemap,
};

pub enum GameScreen {
    Main,
    Log { offset: usize },
    Examine { cursor: Position },
}

#[derive(Clone)]
pub struct CharWidget {
    position: Position,
    renderable: Renderable,
}

impl Widget for CharWidget {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        // add and subtract 1 to account for borders
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

pub struct AsciiGauge {
    ratio: f64,
    filled_glyph: char,
    unfilled_glyph: char,
    filled_style: Style,
    unfilled_style: Style,
}

impl AsciiGauge {
    pub fn default() -> Self {
        Self {
            ratio: 0.5,
            filled_glyph: '=',
            unfilled_glyph: '-',
            filled_style: Style::default(),
            unfilled_style: Style::default(),
        }
    }

    pub fn filled_glyph(mut self, glyph: char) -> Self {
        self.filled_glyph = glyph;
        self
    }

    pub fn unfilled_glyph(mut self, glyph: char) -> Self {
        self.unfilled_glyph = glyph;
        self
    }

    pub fn filled_style(mut self, style: Style) -> Self {
        self.filled_style = style;
        self
    }

    pub fn unfilled_style(mut self, style: Style) -> Self {
        self.unfilled_style = style;
        self
    }

    pub fn ratio(mut self, ratio: f64) -> Self {
        self.ratio = ratio;
        self
    }
}

impl Widget for AsciiGauge {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let filled_chars = (area.width as f64 * self.ratio).ceil() as u16;
        for i in 0..filled_chars {
            let cell = &mut buf[(area.x + i, area.y)];
            cell.set_char(self.filled_glyph)
                .set_style(self.filled_style);
        }
        for i in filled_chars..area.width {
            let cell = &mut buf[(area.x + i, area.y)];
            cell.set_char(self.unfilled_glyph)
                .set_style(self.unfilled_style);
        }
    }
}

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let horizontal_split = layout::Layout::default()
            .direction(layout::Direction::Horizontal)
            .constraints(vec![
                layout::Constraint::Min(15),
                layout::Constraint::Percentage(70),
            ])
            .split(frame.area());

        let ui_layout = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(vec![
                layout::Constraint::Percentage(30),
                layout::Constraint::Percentage(70),
            ])
            .split(horizontal_split[0]);

        let world_layout = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(vec![
                layout::Constraint::Percentage(70),
                layout::Constraint::Min(5),
            ])
            .split(horizontal_split[1]);

        // correct the offset before it gets passed to render fullscreen log
        match &mut self.game_screen {
            GameScreen::Log { offset } => {
                let display_idx = self
                    .log
                    .len()
                    .saturating_sub(horizontal_split[1].height as usize - 2);
                *offset = (*offset).min(display_idx);
            }
            _ => {}
        }

        match self.game_screen {
            GameScreen::Main => {
                self.render_map(frame, world_layout[0]);
                self.render_entities(frame, world_layout[0]);
                self.render_log(frame, world_layout[1]);

                self.render_status(frame, ui_layout[0]);
                self.render_inventory(frame, ui_layout[1]);
            }
            GameScreen::Log { offset } => {
                self.render_fullscreen_log(frame, horizontal_split[1], offset);
                self.render_status(frame, ui_layout[0]);
                self.render_inventory(frame, ui_layout[1]);
            }
            GameScreen::Examine { cursor } => {
                self.render_map(frame, world_layout[0]);
                self.render_entities(frame, world_layout[0]);
                self.render_examine_cursor(frame, world_layout[0], &cursor);
                self.render_examine_info(frame, world_layout[1], &cursor);

                self.render_status(frame, ui_layout[0]);
                self.render_inventory(frame, ui_layout[1]);
            }
        }
    }

    /// render tiles in gamemap
    fn render_map(&self, frame: &mut Frame, area: layout::Rect) {
        let inner_area = area.inner(layout::Margin {
            horizontal: 1,
            vertical: 1,
        });
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
                frame.render_widget(ch, inner_area);
            }
        }
    }

    /// render the cursor in the map after rendering everything else
    fn render_examine_cursor(&self, frame: &mut Frame, area: layout::Rect, cursor: &Position) {
        // use inner_area because render_map() also renders to this
        let inner_area = area.inner(layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        // swap the fg and bg colors of the cell the cursor is highlighting
        let buf = frame.buffer_mut();
        let coords = (cursor.x + inner_area.x, cursor.y + inner_area.y);
        let cell = &mut buf[coords];

        let (fg, bg) = (cell.fg, cell.bg);

        if bg == Color::Reset {
            cell.set_fg(Color::Black);
        } else {
            cell.set_fg(bg);
        }

        if fg == Color::default() {
            cell.set_bg(Color::Gray);
        } else {
            cell.set_bg(fg);
        }
    }

    /// displays information about the examined item
    fn render_examine_info(&self, frame: &mut Frame, area: layout::Rect, cursor: &Position) {
        // find all the objects located at the cursor
        let mut names = Vec::new();
        for id in self.gamemap.object_ids.iter() {
            let obj = self.objects.get(id).unwrap();
            if &obj.pos == cursor {
                names.push(&obj.name);
            }
        }

        let mut lines: Vec<Line> = Vec::new();
        lines.push(format!("Things here:").into());

        if names.len() == 0 {
            lines.push(format!(" - the floor.").into());
        } else {
            for name in names {
                lines.push(format!(" - {}", name).into());
            }
        }

        let paragraph =
            Paragraph::new(lines).block(Block::default().title("examine").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }

    /// render all objects in the gamemap to screen
    fn render_entities(&self, frame: &mut Frame, area: layout::Rect) {
        let block = Block::default().title("world").borders(Borders::ALL);
        frame.render_widget(block, area);
        let inner_area = area.inner(layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        let mut indices_to_draw = self.gamemap.object_ids.clone();
        indices_to_draw.sort_by(|a, b| {
            let obj_a = self.objects.get(&a).unwrap();
            let obj_b = self.objects.get(&b).unwrap();
            obj_a.blocks_movement.cmp(&obj_b.blocks_movement)
        });

        for obj in indices_to_draw
            .iter()
            .map(|id| self.objects.get(id).unwrap())
        {
            let position = &obj.pos;
            let renderable = &obj.renderable;

            let ch = CharWidget {
                position: position.clone(),
                renderable: renderable.clone(),
            };

            match obj.render_status {
                RenderStatus::Hide => {}
                RenderStatus::ShowInFOV => {
                    // render only visible entities
                    if self.gamemap.is_visible(position.x, position.y) {
                        frame.render_widget(ch, inner_area);
                    }
                }
                RenderStatus::AlwaysShow => {
                    frame.render_widget(ch, inner_area);
                }
            }
        }
    }

    /// renders the text in the log
    fn render_log(&self, frame: &mut Frame, area: layout::Rect) {
        let mut lines: Vec<Line> = self.log.iter().map(|s| Line::from(s.as_str())).collect();
        let display_idx = lines.len().saturating_sub(area.height as usize - 2);
        let lines_to_render = lines.split_off(display_idx);

        let paragraph = Paragraph::new(lines_to_render)
            .block(Block::default().title("log").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }

    /// renders log text with offset to the fullscreen log viewer
    /// returns the given offset clamped to be in bounds
    fn render_fullscreen_log(&self, frame: &mut Frame, area: layout::Rect, offset: usize) {
        let mut lines: Vec<Line> = self.log.iter().map(|s| Line::from(s.as_str())).collect();

        let split_idx = lines
            .len()
            .saturating_sub(area.height as usize + offset - 2);

        let _overflow_lines = lines.split_off(lines.len() - offset); // delete the bottom offset lines
        let lines_to_render = lines.split_off(split_idx); // split off enough lines to fill the log

        let paragraph = Paragraph::new(lines_to_render)
            .block(Block::default().title("log").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }

    /// renders inventory ui on the left side of the screen
    fn render_status(&self, frame: &mut Frame, area: layout::Rect) {
        let block = Block::default().title("character").borders(Borders::ALL);
        frame.render_widget(block, area);

        let inner_area = area.inner(layout::Margin::new(1, 1));
        let layout = layout::Layout::default()
            .direction(layout::Direction::Horizontal)
            .constraints(vec![
                layout::Constraint::Length(12),
                layout::Constraint::Percentage(100),
            ])
            .split(inner_area);

        let label_area = layout[0];
        let gauge_area = layout[1];

        let player = &self.objects.get(&PLAYER).unwrap();
        let fighter = &player.fighter.as_ref().unwrap();
        let ratio = fighter.hp as f64 / fighter.max_hp as f64;

        let label_text = format!("HP: {}/{}", fighter.hp, fighter.max_hp);
        let health_label = Paragraph::new(label_text);

        let health_gauge = AsciiGauge::default()
            .ratio(ratio)
            .filled_style(Style::default().fg(Color::Green))
            .unfilled_style(Style::default().fg(Color::Red));

        frame.render_widget(health_label, label_area);
        frame.render_widget(health_gauge, gauge_area);
    }

    fn render_inventory(&self, frame: &mut Frame, area: layout::Rect) {
        let block = Block::default().title("inventory").borders(Borders::ALL);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();
        let mut index = 1;
        for id in &self.inventory {
            lines.push(Line::from(format!(
                "({}) {}",
                index,
                self.objects.get(id).unwrap().name
            )));
            index += 1;
        }

        if self.inventory.len() == 0 {
            lines.push(Line::from("inventory is empty."));
        }

        let paragraph = Paragraph::new(lines);
        let inner_area = area.inner(layout::Margin {
            horizontal: 1,
            vertical: 1,
        });
        frame.render_widget(paragraph, inner_area);
    }
}
