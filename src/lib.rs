use std::collections::{HashMap, HashSet, VecDeque};

use wasmbot_client::*;
use wasmbot_messages::{
    Direction, InitialParameters, Message, MoveResult, MoveTo, Open, Point, PresentCircumstances,
    TileType,
};

struct Mapper {
    map: HashMap<(i16, i16), (TileType)>,
    pos: (i16, i16),
    last_move: Option<Direction>,
}
impl Client for Mapper {
    fn create() -> Self {
        return Mapper {
            map: HashMap::new(),
            pos: (0, 0),
            last_move: None,
        };
    }

    fn receive_game_params(&mut self, _params: InitialParameters) -> bool {
        return true;
    }

    fn get_metadata(&mut self) -> params::BotMetadata {
        return params::BotMetadata {
            name: params::make_bot_name("Mapper"),
            version: params::parse_bot_version(env!("CARGO_PKG_VERSION")).unwrap_or([0; 3]),
        };
    }

    // NOTE if this panics it silently fails, no user feedback, TODO fix this in the rust wasmbot library
    fn tick(&mut self, pc: PresentCircumstances) -> Message {
        self.update_pos(pc.last_move_result);

        if let Some(action) = open_door_if_near(&pc) {
            return Message::Open(action);
        }

        self.map_tiles(&pc);

        let direction = self.pathfind();

        self.last_move = Some(direction);

        return Message::MoveTo(MoveTo {
            direction,
            distance: 1,
        });
    }
}

impl Mapper {
    fn update_pos(&mut self, result: MoveResult) {
        if result == MoveResult::Succeeded {
            if self.last_move.is_none() {
                return;
            }
            let last_move = self.last_move.unwrap();
            if last_move == Direction::North {
                self.pos = (self.pos.0, self.pos.1 - 1);
            } else if last_move == Direction::South {
                self.pos = (self.pos.0, self.pos.1 + 1);
            } else if last_move == Direction::West {
                self.pos = (self.pos.0 - 1, self.pos.1);
            } else if last_move == Direction::East {
                self.pos = (self.pos.0 + 1, self.pos.1);
            }
        }
    }

    fn map_tiles(&mut self, pc: &PresentCircumstances) {
        let radius: i16 = pc.surroundings_radius as i16;
        let width: i16 = radius * 2 + 1;
        let total_size: i16 = width * width;

        for i in 0..total_size {
            let x: i16 = (i % width) - radius + self.pos.0;
            let y: i16 = (i / width) - radius + self.pos.1;

            let tile = pc.surroundings[i as usize];
            if tile == TileType::Void {
                continue;
            }
            // add to the map overrides if it was already there, need to override for void tiles
            self.map.insert((x, y), tile);
        }
    }

    fn pathfind(&self) -> Direction {
        let mut visited: HashSet<(i16, i16)> = HashSet::new();
        let mut frontier: VecDeque<((i16, i16), Vec<Direction>)> = VecDeque::new();

        frontier.push_back((self.pos, vec![]));

        while let Some(a) = frontier.pop_front() {
            visited.insert(a.0);

            let (up, down, left, right) = adjacents(&a.0);

            // the thing -----
            let thing = self.map.get(&up);
            if thing.is_none() || thing.unwrap() == &TileType::Void {
                // NOTE WIN, return a direction
                // log(&format!("going to {:?} and {:?}", a.0, a.1));
                return a.1[0];
            }

            let thing = thing.unwrap();

            // if traversible
            if !visited.contains(&up)
                && (thing == &TileType::Floor
                    || thing == &TileType::OpenDoor
                    || thing == &TileType::ClosedDoor)
            {
                let mut new_list = a.1.clone();
                new_list.push(Direction::North);
                frontier.push_back((up, new_list));
            }

            // the thing -----
            let thing = self.map.get(&down);
            if thing.is_none() || thing.unwrap() == &TileType::Void {
                // NOTE WIN, return a direction
                // log(&format!("going to {:?} and {:?}", a.0, a.1));
                return a.1[0];
            }

            let thing = thing.unwrap();

            // if traversible
            if !visited.contains(&down)
                && (thing == &TileType::Floor
                    || thing == &TileType::OpenDoor
                    || thing == &TileType::ClosedDoor)
            {
                let mut new_list = a.1.clone();
                new_list.push(Direction::South);
                frontier.push_back((down, new_list));
            }

            // the thing -----
            let thing = self.map.get(&left);
            if thing.is_none() || thing.unwrap() == &TileType::Void {
                // NOTE WIN, return a direction
                // log(&format!("going to {:?} and {:?}", a.0, a.1));
                return a.1[0];
            }

            let thing = thing.unwrap();

            // if traversible
            if !visited.contains(&left)
                && (thing == &TileType::Floor
                    || thing == &TileType::OpenDoor
                    || thing == &TileType::ClosedDoor)
            {
                let mut new_list = a.1.clone();
                new_list.push(Direction::West);
                frontier.push_back((left, new_list));
            }

            // the thing -----
            let thing = self.map.get(&right);
            if thing.is_none() || thing.unwrap() == &TileType::Void {
                // NOTE WIN, return a direction
                // log(&format!("going to {:?} and {:?}", a.0, a.1));
                return a.1[0];
            }

            let thing = thing.unwrap();

            // if traversible
            if !visited.contains(&right)
                && (thing == &TileType::Floor
                    || thing == &TileType::OpenDoor
                    || thing == &TileType::ClosedDoor)
            {
                let mut new_list = a.1.clone();
                new_list.push(Direction::East);
                frontier.push_back((right, new_list));
            }
        }

        log("died");
        panic!();
    }
}

fn open_door_if_near(pc: &PresentCircumstances) -> Option<Open> {
    let radius: i16 = pc.surroundings_radius as i16;
    let width: i16 = radius * 2 + 1;
    let total_size: i16 = width * width;

    for i in 0..total_size {
        let x: i16 = (i % width) - radius;
        let y: i16 = (i / width) - radius;

        // check if in range
        if !((x == 0 && (y == 1 || y == -1)) || (y == 0 && (x == 1 || x == -1))) {
            continue;
        }

        if pc.surroundings[i as usize] != TileType::ClosedDoor {
            continue;
        }

        // add to the map if it is not yet mapped
        return Some(Open {
            target: Point { x, y },
        });
    }
    return None;
}

#[rustfmt::skip]
fn adjacents(pos: &(i16, i16)) -> ((i16, i16), (i16, i16), (i16, i16), (i16, i16)) {
    let up    = ( pos.0,     pos.1 - 1 );
    let down  = ( pos.0,     pos.1 + 1 );
    let left  = ( pos.0 - 1, pos.1     );
    let right = ( pos.0 + 1, pos.1     );

    return (up, down, left, right);
}

register_client!(Mapper);
