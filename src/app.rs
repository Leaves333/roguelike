use crate::{entities, gamemap::GameMap, procgen::generate_dungeon};
use hecs::{Entity, World};

mod event_handler;
mod render;

pub struct App {
    gamemap: GameMap,
    player: Entity,
    log: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        let mut world = World::new();
        let player = world.spawn(entities::player(0, 0));
        let log = Vec::new();

        let max_rooms = 30;
        let room_min_size = 6;
        let room_max_size = 10;
        let max_monsters_per_room = 1;

        let dungeon_width = 80;
        let dungeon_height = 24;

        let mut gamemap = generate_dungeon(
            max_rooms,
            room_min_size,
            room_max_size,
            max_monsters_per_room,
            dungeon_width,
            dungeon_height,
            world,
            player,
        );

        gamemap.update_fov(player, 8);
        Self {
            gamemap,
            player,
            log,
        }
    }
}
