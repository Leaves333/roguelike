use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn default() -> Self {
        Position { x: 0, y: 0 }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
    pub bg: Color,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RenderStatus {
    Hide,
    ShowInFOV,
    ShowInExplored,
}

// NOTE: enums are ordered by their discriminants. discriminants are smallest for values at the top
// see https://doc.rust-lang.org/std/cmp/trait.Ord.html

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderLayer {
    Corpse,
    Item,
    Blocking,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Object {
    pub name: String,
    pub pos: Position,
    pub renderable: Renderable,
    pub render_status: RenderStatus,
    pub render_layer: RenderLayer,
    pub blocks_movement: bool,
    pub alive: bool,
    pub fighter: Option<Fighter>,
    pub ai: Option<AIType>,
    pub item: Option<Item>,
    pub equipment: Option<Equipment>,
}

impl Object {
    /// constructs a new object with default position. sets all Option<_> fields to None by default.
    pub fn new(
        name: String,
        renderable: Renderable,
        render_status: RenderStatus,
        render_layer: RenderLayer,
        blocks_movement: bool,
        alive: bool,
    ) -> Self {
        Self {
            name,
            pos: Position::default(),
            renderable,
            render_layer,
            render_status,
            blocks_movement,
            alive,
            fighter: None,
            ai: None,
            item: None,
            equipment: None,
        }
    }

    pub fn set_fighter(mut self, fighter: Fighter) -> Self {
        self.fighter = Some(fighter);
        self
    }

    pub fn set_ai(mut self, ai: AIType) -> Self {
        self.ai = Some(ai);
        self
    }

    pub fn set_item(mut self, item: Item) -> Self {
        self.item = Some(item);
        self
    }

    pub fn set_equipment(mut self, equipment: Equipment) -> Self {
        self.equipment = Some(equipment);
        self
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Fighter {
    pub max_hp: u16,
    pub hp: u16,
    pub defense: i16,
    pub power: i16,
    pub death_callback: DeathCallback,
}

impl Fighter {
    pub fn new(max_hp: u16, defense: i16, power: i16, death_callback: DeathCallback) -> Self {
        Self {
            max_hp,
            hp: max_hp,
            defense,
            power,
            death_callback,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AIType {
    Melee,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DeathCallback {
    Player,
    Monster,
}

/// represents information about an item.
/// should not store persistent data, as this will get cloned
#[derive(Clone, Serialize, Deserialize)]
pub enum Item {
    Heal,
    Lightning,
    Equipment,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Slot {
    Weapon = 0,
    Head = 1,
    Body = 2,
}
pub const SLOT_ORDERING: [Slot; 3] = [Slot::Weapon, Slot::Head, Slot::Body];

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Weapon => {
                write!(f, "Weapon")
            }
            Slot::Head => {
                write!(f, "Head")
            }
            Slot::Body => {
                write!(f, "Body")
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Equipment {
    pub slot: Slot,
    // pub power_bonus: u16,
    // pub defense_bonus: u16,
}
