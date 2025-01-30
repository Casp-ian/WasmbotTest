use std::collections::HashMap;

use wasmbot_client::*;
use wasmbot_messages::{Direction, Point, TileType};

struct Mapper {
    data: HashMap<(i16, i16), TileType>,
    pos: (i16, i16),
    path: Vec<Direction>,
}
impl Client for Mapper {
    fn create() -> Self {
        return Mapper {
            data: HashMap::new(),
            pos: (0, 0),
            path: vec![],
        };
    }

    fn receive_game_params(&mut self, _params: wasmbot_messages::InitialParameters) -> bool {
        return true;
    }

    fn get_metadata(&mut self) -> params::BotMetadata {
        return params::BotMetadata {
            name: params::make_bot_name("Mapper"),
            version: params::parse_bot_version(env!("CARGO_PKG_VERSION")).unwrap_or([0; 3]),
        };
    }

    // NOTE if this panics it silently fails, no user feedback, TODO fix this in the rust wasmbot library
    fn tick(&mut self, pc: wasmbot_messages::PresentCircumstances) -> wasmbot_messages::Message {
        if let Some(action) = open_door(&pc) {
            return wasmbot_messages::Message::Open(action);
        }
        remember_tiles(&self.pos, &mut self.data, &pc);
        pathfind(&self.pos, &mut self.data, &(0, 0));

        return wasmbot_messages::Message::MoveTo(wasmbot_messages::MoveTo {
            direction: wasmbot_messages::Direction::North,
            distance: 1,
        });
    }
}

fn open_door(pc: &wasmbot_messages::PresentCircumstances) -> Option<wasmbot_messages::Open> {
    let radius: i16 = pc.surroundings_radius as i16;
    let width: i16 = radius * 2 + 1;
    let total_size: i16 = width * width;

    for i in 0..total_size {
        let x: i16 = (i % width) - radius;
        let y: i16 = (i / width) - radius;

        // check if in range
        if !(x <= -1 && x >= 1 && y <= -1 && y >= 1) {
            continue;
        }

        if pc.surroundings[i as usize] != TileType::ClosedDoor {
            continue;
        }
        // add to the map if it is not yet mapped
        return Some(wasmbot_messages::Open {
            target: Point { x, y },
        });
    }
    return None;
}

fn remember_tiles(
    pos: &(i16, i16),
    map: &mut HashMap<(i16, i16), TileType>,
    pc: &wasmbot_messages::PresentCircumstances,
) -> () {
    let radius: i16 = pc.surroundings_radius as i16;
    let width: i16 = radius * 2 + 1;
    let total_size: i16 = width * width;

    for i in 0..total_size {
        let x: i16 = (i % width) - radius + pos.0;
        let y: i16 = (i / width) - radius + pos.1;

        // add to the map if it is not yet mapped
        if !map.contains_key(&(x, y)) {
            map.insert((x, y), pc.surroundings[i as usize]);
        }
    }
}

fn pathfind(
    pos: &(i16, i16),
    map: &mut HashMap<(i16, i16), TileType>,
    goal: &(i16, i16),
) -> Vec<Direction> {
    // TODO astar or equivalent
    todo!();
}

register_client!(Mapper);
