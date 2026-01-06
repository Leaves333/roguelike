use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{self, Constraint, Direction, Flex, Layout, Margin, Rect},
    style::{Color, Style, Styled, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

use super::{App, GameScreen, PLAYER};
use crate::{
    components::{Position, Renderable, SLOT_ORDERING},
    engine::{defense, power},
    gamemap::{self, Tile, TileType},
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

pub struct AsciiGauge {
    ratio: f64,
    filled_glyph: char,
    unfilled_glyph: char,
    filled_style: Style,
    unfilled_style: Style,
}

#[allow(dead_code)]
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

    pub fn set_filled_glyph(mut self, glyph: char) -> Self {
        self.filled_glyph = glyph;
        self
    }

    pub fn set_unfilled_glyph(mut self, glyph: char) -> Self {
        self.unfilled_glyph = glyph;
        self
    }

    pub fn set_filled_style(mut self, style: Style) -> Self {
        self.filled_style = style;
        self
    }

    pub fn set_unfilled_style(mut self, style: Style) -> Self {
        self.unfilled_style = style;
        self
    }

    pub fn set_ratio(mut self, ratio: f64) -> Self {
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

/// creates a Rect that is centered in area based on the horizontal and vertical constraints
fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

/// converts the given time to a string
/// used to consistently format time in different locations
fn time_string(time: u64) -> String {
    format!("{:<5.1}", (time as f64) / 100.0)
}

/// computes offset x and y relative to a center location and the center of the area
/// used for rendering objects to the worldmap
fn relative_coords(area: Rect, center_pos: Position, target_pos: Position) -> Option<Position> {
    let center = Position {
        x: area.width / 2,
        y: area.height / 2,
    };

    let x = match (center.x + target_pos.x).checked_sub(center_pos.x) {
        Some(x) => x,
        None => {
            return None;
        }
    };
    let y = match (center.y + target_pos.y).checked_sub(center_pos.y) {
        Some(y) => y,
        None => {
            return None;
        }
    };

    if x > area.width || y > area.height {
        None
    } else {
        Some(Position { x, y })
    }
}

/// returns the way that a tile will appear on the map,
/// based on what items/blockers are on top of it
pub fn tile_topmost_renderable(app: &App, tile: &Tile) -> Renderable {
    if let Some(blocker_id) = tile.blocker {
        let blocker = app.objects.get(&blocker_id).unwrap();
        return blocker.renderable.clone();
    }
    if let Some(item_id) = tile.item {
        let item = app.objects.get(&item_id).unwrap();
        return item.renderable.clone();
    }
    tile.renderable()
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
                layout::Constraint::Percentage(30),
                layout::Constraint::Percentage(40),
            ])
            .split(horizontal_split[0]);

        let world_layout = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(vec![
                layout::Constraint::Percentage(70),
                layout::Constraint::Min(5),
            ])
            .split(horizontal_split[1]);

        // correct game screen variables before they get rendered
        // need to do this first because game_screen needs to be borrowed as mut
        match &mut self.game_screen {
            GameScreen::Log { offset } => {
                // correct the offset before it gets passed to render fullscreen log
                let display_idx = self
                    .log
                    .len()
                    .saturating_sub(horizontal_split[1].height as usize - 2);
                *offset = (*offset).min(display_idx);
            }
            GameScreen::Examine { cursor } | GameScreen::Targeting { cursor, .. } => {
                // keep the cursor within bounds of the renderable area
                let inner_area = world_layout[0].inner(Margin {
                    horizontal: 1,
                    vertical: 1,
                });
                let center = Position {
                    x: inner_area.width / 2,
                    y: inner_area.height / 2,
                };
                let player_pos = self.gamemap.get_position(PLAYER).unwrap();

                match (player_pos.x + inner_area.width).checked_sub(center.x) {
                    Some(x) => {
                        cursor.x = cursor.x.min(x);
                    }
                    None => {}
                }
                match player_pos.x.checked_sub(center.x) {
                    Some(x) => {
                        cursor.x = cursor.x.max(x);
                    }
                    None => {}
                }
                match (player_pos.y + inner_area.height).checked_sub(center.y) {
                    Some(y) => {
                        cursor.y = cursor.y.min(y);
                    }
                    None => {}
                }
                match player_pos.y.checked_sub(center.y) {
                    Some(y) => {
                        cursor.y = cursor.y.max(y);
                    }
                    None => {}
                }
            }
            _ => {}
        }

        // left side status + inventory is rendered on all game screens except the main menu
        match self.game_screen {
            GameScreen::Menu => {}
            _ => {
                let status_area = ui_layout[0];
                let equipment_area = ui_layout[1];
                let inventory_area = ui_layout[2];
                self.render_status(frame, status_area);
                self.render_equipment(frame, equipment_area);
                self.render_inventory(frame, inventory_area);
            }
        }

        match self.game_screen {
            GameScreen::Menu => {
                self.render_main_menu(frame, frame.area());
            }
            GameScreen::Main => {
                self.render_tiles(frame, world_layout[0]);
                self.render_log(frame, world_layout[1]);
            }
            GameScreen::Log { offset } => {
                self.render_fullscreen_log(frame, horizontal_split[1], offset);
            }
            GameScreen::Examine { ref cursor } => {
                self.render_tiles(frame, world_layout[0]);

                self.render_examine_cursor(frame, world_layout[0], &cursor);
                self.render_examine_info(frame, world_layout[1], &cursor);
            }
            GameScreen::Targeting {
                ref cursor,
                ref text,
                ..
            } => {
                self.render_tiles(frame, world_layout[0]);

                // TODO: render targeting line of fire overlay to world map
                self.render_examine_cursor(frame, world_layout[0], &cursor);
                self.render_targeting_info(frame, world_layout[1], &cursor, text);
            }
        }
    }

    fn render_main_menu(&self, frame: &mut Frame, area: layout::Rect) {
        // render border in middle of screen
        let inner = center(area, Constraint::Percentage(50), Constraint::Percentage(50));
        let block = Block::default().title("menu").borders(Borders::ALL);
        frame.render_widget(block, inner);

        let inner = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        let title_lines: Vec<Line> = vec![
            Line::from("epic cool game title :DDD").set_style(Style::new().bold()),
            Line::from("by epic cool guy"),
        ];
        let instruction_lines: Vec<Line> = vec![
            Line::from("(n) New Game"),
            Line::from("(l) Load Game"),
            Line::from("(q) Quit"),
        ];

        let [title_area, _, instruction_area] = Layout::vertical([
            Constraint::Length(title_lines.len() as u16),
            Constraint::Length(3), // magic number for padding between the two areas
            Constraint::Length(instruction_lines.len() as u16),
        ])
        .flex(Flex::Center)
        .areas(inner);

        // magic number for the length of the instruction text
        let [instruction_area] = Layout::horizontal([Constraint::Length(15)])
            .flex(Flex::Center)
            .areas(instruction_area);

        let title_paragraph = Paragraph::new(title_lines).centered();
        let instruction_paragraph = Paragraph::new(instruction_lines);
        frame.render_widget(title_paragraph, title_area);
        frame.render_widget(instruction_paragraph, instruction_area);
    }

    /// render tiles in gamemap
    fn render_tiles(&self, frame: &mut Frame, area: layout::Rect) {
        let inner_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        // cover inner area in dark tiles
        for x in 0..area.width {
            for y in 0..area.height {
                let ch = CharWidget {
                    position: Position { x, y },
                    renderable: Renderable {
                        glyph: '.',
                        fg: Color::Black,
                        bg: Color::Reset,
                    },
                };
                frame.render_widget(ch, inner_area);
            }
        }

        // render the tiles in the gamemap
        let player_pos = self.gamemap.get_position(PLAYER).unwrap();
        for x in 0..self.gamemap.width {
            for y in 0..self.gamemap.height {
                let target_pos = match relative_coords(inner_area, player_pos, Position { x, y }) {
                    Some(pos) => pos,
                    None => {
                        continue;
                    }
                };

                let tile = self.gamemap.get_ref(x, y);
                let ch = CharWidget {
                    position: target_pos,
                    renderable: {
                        if self.gamemap.is_visible(x, y) {
                            tile_topmost_renderable(self, tile)
                        } else if self.gamemap.is_explored(x, y) {
                            let last_seen = self.gamemap.get_last_seen(x, y);
                            Renderable {
                                glyph: last_seen.glyph,
                                fg: Color::DarkGray,
                                bg: Color::Reset,
                            }
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
    fn render_examine_cursor(&self, frame: &mut Frame, area: Rect, cursor: &Position) {
        // use inner_area because render_map() also renders to this
        let inner_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        // swap the fg and bg colors of the cell the cursor is highlighting
        let player_pos = self.gamemap.get_position(PLAYER).unwrap();
        let offset_pos = match relative_coords(
            inner_area,
            player_pos,
            Position {
                x: cursor.x,
                y: cursor.y,
            },
        ) {
            Some(pos) => pos,
            None => {
                panic!("um.");
            }
        };
        let coords = (inner_area.x + offset_pos.x, inner_area.y + offset_pos.y);
        let buf = frame.buffer_mut();
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

    /// displays information about items under the examine cursor
    fn render_examine_info(&self, frame: &mut Frame, area: Rect, cursor: &Position) {
        let lines: Vec<Line> = self
            .get_objects_at_cursor(cursor)
            .into_iter()
            .map(|x| Line::from(x))
            .collect();
        let paragraph =
            Paragraph::new(lines).block(Block::default().title("examine").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }

    /// displays the targeting info box.
    /// works like render_examine_info, but with an extra line about what you are targeting
    fn render_targeting_info(&self, frame: &mut Frame, area: Rect, cursor: &Position, text: &str) {
        let mut lines = vec![Line::from(text)];
        lines.extend(
            self.get_objects_at_cursor(cursor)
                .into_iter()
                .map(|x| Line::from(x)),
        );
        let paragraph =
            Paragraph::new(lines).block(Block::default().title("targeting").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }

    /// returns a vec containing the names of objects at the cursor
    /// to be used with render_examine() and render_targeting()
    fn get_objects_at_cursor(&self, cursor: &Position) -> Vec<String> {
        let mut names = Vec::new();

        let tile = self.gamemap.get_ref(cursor.x, cursor.y);
        if let Some(id) = tile.blocker {
            let obj = self.objects.get(&id).unwrap();
            if &self.gamemap.get_position(id).unwrap() == cursor {
                names.push(&obj.name);
            }
        }
        if let Some(id) = tile.item {
            let obj = self.objects.get(&id).unwrap();
            if &self.gamemap.get_position(id).unwrap() == cursor {
                names.push(&obj.name);
            }
        }

        let mut formatted: Vec<String> = Vec::new();
        formatted.push(format!("Things here:").into());

        if names.len() == 0 {
            if !self.gamemap.is_visible(cursor.x, cursor.y) {
                formatted.push(format!("   you can't see this tile.").into());
            } else {
                let tile = self.gamemap.get_ref(cursor.x, cursor.y);
                if *tile == Tile::new(TileType::Floor) {
                    formatted.push(format!("   the floor.").into());
                } else if *tile == Tile::new(TileType::Wall) {
                    formatted.push(format!("   a wall.").into());
                }
            }
        } else {
            for name in names {
                formatted.push(format!("   {}", name).into());
            }
        }

        formatted
    }

    /// converts the log into a list of lines,
    /// used in `render_log` / `render_fullscreen_log`
    fn get_lines_from_log(&self) -> Vec<Line> {
        self.log
            .iter()
            .map(|entry| {
                Line::from(format!(
                    "{} {}",
                    time_string(entry.time),
                    entry.message.as_str()
                ))
                .style(entry.style)
            })
            .collect()
    }

    /// renders the text in the log
    fn render_log(&self, frame: &mut Frame, area: Rect) {
        let mut lines = self.get_lines_from_log();
        let display_idx = lines.len().saturating_sub(area.height as usize - 2);
        let lines_to_render = lines.split_off(display_idx);

        let paragraph = Paragraph::new(lines_to_render)
            .block(Block::default().title("log").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }

    /// renders log text with offset to the fullscreen log viewer
    /// returns the given offset clamped to be in bounds
    fn render_fullscreen_log(&self, frame: &mut Frame, area: Rect, offset: usize) {
        let mut lines = self.get_lines_from_log();
        let split_idx = lines
            .len()
            .saturating_sub(area.height as usize + offset - 2);

        let _overflow_lines = lines.split_off(lines.len() - offset); // delete the bottom offset lines
        let lines_to_render = lines.split_off(split_idx); // split off enough lines to fill the log

        let paragraph = Paragraph::new(lines_to_render)
            .block(Block::default().title("log").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }

    /// renders healthbar and stats on the left side of the screen
    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title("character").borders(Borders::ALL);
        frame.render_widget(block, area);

        // first split the area vertically
        let inner_area = area.inner(Margin::new(1, 1));
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Percentage(100),
            ])
            .split(inner_area);

        let gauges_area = layout[0]; // for health and mana gauges
        let dungeon_area = layout[1]; // for displaying time and dungeon depth
        let stats_area = layout[2]; // for displaying player stats

        // render health bar gauge on top most area
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(12), Constraint::Percentage(100)])
            .split(gauges_area);
        let label_area = layout[0];
        let gauge_area = layout[1];

        let player = &self.objects.get(&PLAYER).unwrap();
        let fighter = &player.fighter.as_ref().unwrap();
        let ratio = fighter.hp as f64 / fighter.max_hp as f64;

        let label_text = format!("HP: {}/{}", fighter.hp, fighter.max_hp);
        let health_label = Paragraph::new(label_text);

        let health_gauge = AsciiGauge::default()
            .set_ratio(ratio)
            .set_filled_style(Style::default().fg(Color::Green))
            .set_unfilled_style(Style::default().fg(Color::Red));

        frame.render_widget(health_label, label_area);
        frame.render_widget(health_gauge, gauge_area);

        // render dungeon stats in the middle

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(dungeon_area);
        let time_area = layout[0];
        let depth_area = layout[1];

        let time_line = Line::from(format!("Time: {}", time_string(self.time)));
        let depth_line = Line::from(format!("Depth: {:0>2}", self.gamemap.level));
        let time_paragraph = Paragraph::new(vec![time_line]);
        let depth_paragraph = Paragraph::new(vec![depth_line]).right_aligned();

        frame.render_widget(time_paragraph, time_area);
        frame.render_widget(depth_paragraph, depth_area);

        // render player stats on bottom
        let lines: Vec<Line> = vec![
            Line::from(format!("ATK {}", power(self, PLAYER))),
            Line::from(format!("DEF {}", defense(self, PLAYER))),
        ];
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, stats_area);
    }

    fn render_equipment(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title("equipment").borders(Borders::ALL);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();

        let chars = ["A", "B", "C"];
        let mut index = 0;

        // check: assert that the char array for equipment slot labels matches up with the actual
        // number of slots
        assert_eq!(chars.len(), SLOT_ORDERING.len());

        while index < chars.len() {
            lines.push(Line::from(format!(
                "({}) {:8} {}",
                chars[index],
                format!("{}:", SLOT_ORDERING[index]),
                {
                    match self.equipment[index] {
                        Some(id) => {
                            let obj = self.objects.get(&id).unwrap();
                            obj.name.clone()
                        }
                        None => String::from("(empty)"),
                    }
                }
            )));
            index += 1;
        }

        let paragraph = Paragraph::new(lines);
        let inner_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        frame.render_widget(paragraph, inner_area);
    }

    fn render_inventory(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title("inventory").borders(Borders::ALL);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();
        let mut index = 1;
        for id in &self.inventory {
            lines.push(Line::from(format!(
                "({}) {}",
                index % 10,
                self.objects.get(id).unwrap().name
            )));
            index += 1;
        }

        if self.inventory.len() == 0 {
            lines.push(Line::from("inventory is empty."));
        }

        let paragraph = Paragraph::new(lines);
        let inner_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        frame.render_widget(paragraph, inner_area);
    }
}
