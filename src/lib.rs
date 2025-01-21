use std::collections::HashMap;

use wasmbot_client::*;

struct Mapper {
    data: HashMap<(i32, i32), u8>,
    pos: (i32, i32),
}
impl Client for Mapper {
    fn create() -> Self {
        return Mapper {
            data: HashMap::new(),
            pos: (0, 0),
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

    fn tick(&mut self, pc: wasmbot_messages::PresentCircumstances) -> wasmbot_messages::Message {
        return wasmbot_messages::Message::Wait(wasmbot_messages::Wait {});
    }
}

register_client!(Mapper);
