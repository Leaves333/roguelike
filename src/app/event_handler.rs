use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::style::Color;

use crate::components::SLOT_ORDERING;
use crate::engine::{
    InputDirection, UseResult, bump_action, go_down_stairs, handle_monster_turns, update_fov,
};
use crate::inventory;

use super::procgen::DungeonConfig;
use super::{App, GameScreen, INVENTORY_SIZE, PLAYER, VIEW_RADIUS};

// NOTE: i want this file to contain logic for handling player controls

/// represents the kind of action that a player took
/// NOTE: the Exit variant is here because it impacts the main game loop
/// other actions that only change the state of the app but don't affect the main loop
/// should be handled locally, and not set as a separate enum
enum PlayerAction {
    /// the player took a turn, and their action took u64 time
    TookTurn(u64),
    /// the player didn't take a turn, and we shouldn't increment the time at all
    /// this variant exists to make code more readable
    NoTimeTaken,
    Exit,
}

const PLAYER_MOVEMENT_TIME: u64 = 100;
const PLAYER_ITEM_USE_TIME: u64 = 50;

/// match generic keybinds, used for menu navigation
/// returns a PlayerAction if a keybind was succesfully matched, or None otherwise
fn match_menu_keys(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    match key.modifiers {
        KeyModifiers::CONTROL => match key.code {
            KeyCode::Char('l') => {
                app.toggle_fullscreen_log();
                return Some(PlayerAction::TookTurn(0));
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                return Some(PlayerAction::Exit);
            }
            _ => {}
        },
        _ => match key.code {
            KeyCode::Esc => {
                app.switch_to_main_screen();
                return Some(PlayerAction::TookTurn(0));
            }
            _ => {}
        },
    };
    return None;
}

/// match keybinds for movement
/// returns a PlayerAction if a keybind was succesfully matched, or None otherwise
fn match_movement_keys(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    // movement related controls
    match app.game_screen {
        GameScreen::Main => match key.code {
            // movement keys during the main screen
            KeyCode::Right | KeyCode::Char('l') => {
                bump_action(app, PLAYER, InputDirection::Right);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Left | KeyCode::Char('h') => {
                bump_action(app, PLAYER, InputDirection::Left);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                bump_action(app, PLAYER, InputDirection::Down);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                bump_action(app, PLAYER, InputDirection::Up);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('u') => {
                bump_action(app, PLAYER, InputDirection::UpRight);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('y') => {
                bump_action(app, PLAYER, InputDirection::UpLeft);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('n') => {
                bump_action(app, PLAYER, InputDirection::DownRight);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('b') => {
                bump_action(app, PLAYER, InputDirection::DownLeft);
                return Some(PlayerAction::TookTurn(PLAYER_MOVEMENT_TIME));
            }
            KeyCode::Char('.') => {
                // wait action, nothing is done
                // NOTE: default wait time is 100, independent of player movement speed
                return Some(PlayerAction::TookTurn(100));
            }
            _ => {}
        },
        GameScreen::Examine { ref mut cursor } | GameScreen::Targeting { ref mut cursor, .. } => {
            match key.code {
                // move cursor around during examine and targeting modes
                // do checks to keep cursor within bounds of the gamemap here
                KeyCode::Down | KeyCode::Char('j') => {
                    cursor.y = (cursor.y + 1).min(app.gamemap.height - 1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    cursor.y = cursor.y.saturating_sub(1);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    cursor.x = (cursor.x + 1).min(app.gamemap.width - 1);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    cursor.x = cursor.x.saturating_sub(1);
                }

                KeyCode::Char('u') => {
                    cursor.x = (cursor.x + 1).min(app.gamemap.width - 1);
                    cursor.y = cursor.y.saturating_sub(1);
                }
                KeyCode::Char('y') => {
                    cursor.x = cursor.x.saturating_sub(1);
                    cursor.y = cursor.y.saturating_sub(1);
                }
                KeyCode::Char('n') => {
                    cursor.x = (cursor.x + 1).min(app.gamemap.width - 1);
                    cursor.y = (cursor.y + 1).min(app.gamemap.height - 1);
                }
                KeyCode::Char('b') => {
                    cursor.x = cursor.x.saturating_sub(1);
                    cursor.y = (cursor.y + 1).min(app.gamemap.height - 1);
                }
                _ => {}
            }
        }
        _ => {}
    };
    return None;
}

/// matches controls on the main menu
/// returns a PlayerAction if a keybind was succesfully matched, or None otherwise
fn match_main_menu_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    // check we are on the menu screen
    if app.game_screen != GameScreen::Menu {
        return None;
    }

    match key.code {
        KeyCode::Char('n') => {
            // start new game
            app.new_game();
            app.switch_to_main_screen();
            Some(PlayerAction::NoTimeTaken)
        }
        KeyCode::Char('l') => {
            // loads an existing game from a save file
            let _ = app.load_game();
            app.switch_to_main_screen();
            Some(PlayerAction::NoTimeTaken)
        }
        KeyCode::Char('q') => {
            // quit the game
            Some(PlayerAction::Exit)
        }
        _ => None,
    }
}

fn match_inventory_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    if app.game_screen != GameScreen::Main {
        return None;
    }

    // use alt-number to drop item from inventory
    match key.modifiers {
        KeyModifiers::ALT => {
            match key.code {
                // drop item from inventory
                KeyCode::Char(c @ '1'..='9') | KeyCode::Char(c @ '0') => {
                    let index = match c {
                        '1'..='9' => c as usize - '1' as usize,
                        '0' => 9,
                        _ => unreachable!(),
                    };
                    inventory::drop_item(app, index);
                    return Some(PlayerAction::NoTimeTaken);
                }
                _ => {}
            }
        }
        _ => {}
    }

    match key.code {
        // number keys to use item from inventory
        KeyCode::Char(c @ '1'..='9') | KeyCode::Char(c @ '0') => {
            let index = match c {
                '1'..='9' => c as usize - '1' as usize,
                '0' => 9,
                _ => unreachable!(),
            };

            if app.inventory.len() > index {
                let item = inventory::get_item_in_inventory(app, index).clone();

                if item.needs_targeting() {
                    // item needs targeting, switch to targeting mode
                    item.on_targeting(app, index);
                    return Some(PlayerAction::NoTimeTaken);
                } else {
                    // item can be used directly
                    let use_result = inventory::use_item(app, index, None);
                    return match use_result {
                        UseResult::UsedUp => Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME)),
                        UseResult::Equipped => Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME)),
                        UseResult::Cancelled => Some(PlayerAction::NoTimeTaken),
                    };
                }
            }
        }

        // unequip item from equipment
        KeyCode::Char(c @ 'A'..='C') => {
            let index = c as usize - 'A' as usize;
            match app.equipment[index] {
                Some(id) => {
                    // check we have enough space in inventory to unequip the item
                    if app.inventory.len() >= INVENTORY_SIZE {
                        app.add_to_log("Not enough space in inventory.", Color::default());
                        return Some(PlayerAction::NoTimeTaken);
                    }

                    // unequip and move to inventory
                    app.inventory.push(id);
                    app.equipment[index] = None;
                    return Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME));
                }
                None => {
                    app.add_to_log(
                        format!("No item equipped on {}.", SLOT_ORDERING[index]),
                        Color::default(),
                    );
                    return Some(PlayerAction::NoTimeTaken);
                }
            }
        }

        // `g`rab the first item at player's location
        KeyCode::Char('g') => {
            let player_pos = &app.gamemap.get_position(PLAYER).unwrap();
            let tile = app.gamemap.get_ref(player_pos.x, player_pos.y);
            match tile.item {
                Some(id) => {
                    inventory::pick_item_up(app, id.clone());
                    return Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME));
                }
                None => {
                    return Some(PlayerAction::TookTurn(0));
                }
            }
        }
        _ => {}
    }

    return None;
}

/// matches any remaining game controls on the main screen
fn match_misc_game_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    if app.game_screen != GameScreen::Main {
        return None;
    }

    match key.code {
        // move to examine mode
        KeyCode::Char('x') => {
            app.toggle_examine_mode();
            Some(PlayerAction::NoTimeTaken)
        }

        // go down stairs if stairs exist
        KeyCode::Char('>') => {
            let _ = go_down_stairs(app);
            app.switch_to_main_screen();
            Some(PlayerAction::NoTimeTaken)
        }
        _ => None,
    }
}

fn match_log_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    match app.game_screen {
        GameScreen::Log { ref mut offset } => match key.code {
            KeyCode::PageUp => {
                *offset += 10;
                Some(PlayerAction::NoTimeTaken)
            }
            KeyCode::PageDown => {
                *offset = offset.saturating_sub(10);
                Some(PlayerAction::NoTimeTaken)
            }
            KeyCode::Char('k') => {
                *offset += 1;
                Some(PlayerAction::NoTimeTaken)
            }
            KeyCode::Char('j') => {
                *offset = offset.saturating_sub(1);
                Some(PlayerAction::NoTimeTaken)
            }
            _ => None,
        },
        _ => None,
    }
}

fn match_examine_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    // NOTE: controls for moving the cursor fall under movement controls
    match app.game_screen {
        GameScreen::Examine { .. } => match key.code {
            // exit examine mode
            KeyCode::Char('x') => {
                app.toggle_examine_mode();
                Some(PlayerAction::NoTimeTaken)
            }
            _ => None,
        },
        _ => None,
    }
}

fn match_targeting_controls(app: &mut App, key: KeyEvent) -> Option<PlayerAction> {
    match app.game_screen {
        GameScreen::Targeting {
            ref cursor,
            inventory_idx,
            ..
        } => match key.code {
            KeyCode::Enter => {
                // use the item and exit targeting mode
                let use_result = inventory::use_item(app, inventory_idx, Some(cursor.clone()));
                app.game_screen = GameScreen::Main;

                match use_result {
                    UseResult::UsedUp => Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME)),
                    UseResult::Equipped => Some(PlayerAction::TookTurn(PLAYER_ITEM_USE_TIME)),
                    UseResult::Cancelled => Some(PlayerAction::NoTimeTaken),
                }
            }
            _ => None,
        },
        _ => None,
    }
}

impl App {
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()? {
                let action = self.handle_keys(key);
                match action {
                    PlayerAction::TookTurn(time_taken) => {
                        if time_taken == 0 {
                            continue;
                        }

                        self.time += time_taken;
                        handle_monster_turns(self);
                        update_fov(self, VIEW_RADIUS);
                    }
                    PlayerAction::NoTimeTaken => {
                        continue;
                    }
                    PlayerAction::Exit => {
                        self.save_game()?;
                        break Ok(());
                    }
                }
            }
        }
    }

    /// translate the key event into the appropriate gameplay actions
    fn handle_keys(&mut self, key: KeyEvent) -> PlayerAction {
        let handlers = &[
            match_menu_keys,
            match_movement_keys,
            match_main_menu_controls,
            match_misc_game_controls,
            match_inventory_controls,
            match_log_controls,
            match_examine_controls,
            match_targeting_controls,
        ];

        // iterates through handlers, and gives the first one with a non-none result
        handlers
            .iter()
            .find_map(|handler| handler(self, key))
            .unwrap_or(PlayerAction::NoTimeTaken)
    }

    pub fn new_game(&mut self) {
        self.generate_dungeon(DungeonConfig::default());
        update_fov(self, VIEW_RADIUS);
    }

    fn toggle_fullscreen_log(&mut self) {
        match self.game_screen {
            GameScreen::Log { offset: _ } => self.game_screen = GameScreen::Main,
            _ => self.game_screen = GameScreen::Log { offset: 0 },
        }
    }

    fn toggle_examine_mode(&mut self) {
        match self.game_screen {
            GameScreen::Examine { cursor: _ } => self.game_screen = GameScreen::Main,
            _ => {
                // set default cursor location to player's position
                self.game_screen = GameScreen::Examine {
                    cursor: { self.gamemap.get_position(PLAYER).unwrap() },
                }
            }
        }
    }

    fn switch_to_main_screen(&mut self) {
        self.game_screen = GameScreen::Main;
    }
}
