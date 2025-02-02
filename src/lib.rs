use std::collections::{HashMap, HashSet, VecDeque};

use wasmbot_client::*;
use wasmbot_messages::{
    Direction, InitialParameters, Message, MoveResult, MoveTo, Open, Point, PresentCircumstances,
    TileType,
};

struct Mapper {
    map: HashMap<(i16, i16), TileType>,
    pos: (i16, i16),
    last_move: Option<Direction>,
}
impl Client for Mapper {
    fn create() -> Self {
        let mut map: HashMap<(i16, i16), TileType> = HashMap::new();
        map.reserve(3000);
        return Mapper {
            map,
            pos: (0, 0),
            last_move: None,
        };
    }

    fn receive_game_params(&mut self, params: InitialParameters) -> bool {
        if params.diagonal_movement {
            log("Not built for diagonal movement ;-;");
        }
        return true;
    }

    fn get_metadata(&mut self) -> params::BotMetadata {
        return params::BotMetadata {
            name: params::make_bot_name("Mapper"),
            version: params::parse_bot_version(env!("CARGO_PKG_VERSION")).unwrap_or([0; 3]),
        };
    }

    fn tick(&mut self, pc: PresentCircumstances) -> Message {
        self.update_pos(pc.last_move_result);

        if let Some(action) = open_door_if_near(&pc) {
            self.last_move = None;
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

            // if it already has it, dont remap it
            if self.map.contains_key(&(x, y)) {
                continue;
            }
            // store what tile it is in your map, if it is not a void tile
            let tile = pc.surroundings[i as usize];
            if tile != TileType::Void {
                self.map.insert((x, y), tile);
            }
        }
    }

    // Breadth first search untill first unmapped tile
    fn pathfind(&self) -> Direction {
        let mut visited: HashSet<(i16, i16)> = HashSet::new();
        let mut frontier: VecDeque<((i16, i16), Option<Direction>)> = VecDeque::new();

        frontier.push_back((self.pos, None));

        while let Some(a) = frontier.pop_front() {
            visited.insert(a.0);

            let directions = adjacents(&a.0);
            let directionners = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ];

            for i in 0..4 {
                // skip if direction already visited
                if !visited.contains(&directions[i]) {
                    // if we mapped it already check if traversible
                    if let Some(b) = self.map.get(&directions[i]) {
                        if b != &TileType::Wall {
                            // add to frontier
                            let newa = a.1.unwrap_or(directionners[i]);
                            frontier.push_back((directions[i], Some(newa)));
                        }

                    // if its not yet mapped we go there
                    } else {
                        return a.1.unwrap();
                    }
                }
            }
        }

        log("Cant find any other unmapped tiles!");
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
fn adjacents(pos: &(i16, i16)) -> [(i16, i16); 4] {
    let up    = ( pos.0,     pos.1 - 1 );
    let down  = ( pos.0,     pos.1 + 1 );
    let left  = ( pos.0 - 1, pos.1     );
    let right = ( pos.0 + 1, pos.1     );

    return [up, down, left, right];
}

register_client!(Mapper);
