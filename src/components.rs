use ratatui::style::Color;

#[derive(Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn default() -> Self {
        Position { x: 0, y: 0 }
    }
}

#[derive(Clone)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
    pub bg: Color,
}

#[derive(Clone)]
pub struct Object {
    pub name: String,
    pub pos: Position,
    pub renderable: Renderable,
    pub blocks_movement: bool,
    pub alive: bool,
    pub fighter: Option<Fighter>,
    pub ai: Option<AIType>,
}

impl Object {
    pub fn take_damage(&mut self, damage: u16) {
        // apply damage if possible
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                fighter.hp = fighter.hp.saturating_sub(damage);
            }

            if fighter.hp <= 0 {
                self.alive = false;
                // TODO: death code
                // fighter.on_death.callback(self);
            }

            fighter.hp = fighter.hp.max(fighter.max_hp);
        }
    }
}

#[derive(Clone)]
pub struct Fighter {
    pub max_hp: u16,
    pub hp: u16,
    pub defense: i16,
    pub power: i16,
}

impl Fighter {
    pub fn new(max_hp: u16, defense: i16, power: i16) -> Self {
        Self {
            max_hp,
            hp: max_hp,
            defense,
            power,
        }
    }
}

#[derive(Clone)]
pub enum AIType {
    Melee,
}
