use ratatui::style::Color;

#[derive(Clone, Copy)]
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
pub enum RenderStatus {
    Hide,
    ShowInFOV,
    AlwaysShow,
}

#[derive(Clone)]
pub struct Object {
    pub name: String,
    pub pos: Position,
    pub renderable: Renderable,
    pub render_status: RenderStatus,
    pub blocks_movement: bool,
    pub alive: bool,
    pub fighter: Option<Fighter>,
    pub ai: Option<AIType>,
    pub item: Option<Item>,
}

impl Object {
    /// constructs a new object with default position. sets all Option<_> fields to None by default.
    pub fn new(
        name: String,
        renderable: Renderable,
        render_status: RenderStatus,
        blocks_movement: bool,
        alive: bool,
    ) -> Self {
        Self {
            name,
            pos: Position::default(),
            renderable,
            render_status,
            blocks_movement,
            alive,
            fighter: None,
            ai: None,
            item: None,
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
}

#[derive(Clone)]
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

#[derive(Clone)]
pub enum AIType {
    Melee,
}

#[derive(Clone)]
pub enum DeathCallback {
    Player,
    Monster,
}

#[derive(Clone)]
pub enum Item {
    Heal,
    Lightning,
}
