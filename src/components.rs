use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
    pub bg: Color,
}

impl Renderable {
    pub fn default() -> Self {
        Self {
            glyph: '_',
            fg: Color::default(),
            bg: Color::Reset,
        }
    }
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
    pub name: String,              // name of this object
    pub tooltip: String,           // description of this object
    pub renderable: Renderable,    // how this object looks on the map
    pub render_layer: RenderLayer, // priority on when to render this object
    pub fighter: Option<Fighter>,
    pub ai: Option<AIType>,
    pub item: Option<Item>,
    pub equipment: Option<Equipment>,
}

impl Object {
    /// constructs a new object with default position. sets all Option<_> fields to None by default.
    pub fn new(
        name: String,
        tooltip: String,
        renderable: Renderable,
        render_layer: RenderLayer,
    ) -> Self {
        Self {
            name,
            tooltip,
            renderable,
            render_layer,
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

/// component for objects with health that can be killed
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
    Melee(MeleeAIData),
    Ranged,
}

/// time before melee ai forgets about its target
pub const MELEE_FORGET_TIME: u64 = 500;

#[derive(Clone, Serialize, Deserialize)]
pub struct MeleeAIData {
    pub target: Option<usize>, // id of which object this monster is targeting
    pub last_seen_time: Option<u64>, // when this monster last saw its target
    pub move_speed: u64,       // delay between moves
    pub attack_speed: u64,     // delay between attacks
}

impl MeleeAIData {
    pub fn new() -> Self {
        MeleeAIData {
            target: None,
            last_seen_time: None,
            move_speed: 100,
            attack_speed: 100,
        }
    }

    pub fn set_move_speed(mut self, move_speed: u64) -> Self {
        self.move_speed = move_speed;
        self
    }

    pub fn set_attack_speed(mut self, attack_speed: u64) -> Self {
        self.attack_speed = attack_speed;
        self
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DeathCallback {
    Player,
    Monster,
}

/// represents information about an item.
/// should not store persistent data, as this will get cloned
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Item {
    Heal,
    Lightning,
    Hexbolt,
    Fireball,
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
    pub power_bonus: i16,
    pub defense_bonus: i16,
}
