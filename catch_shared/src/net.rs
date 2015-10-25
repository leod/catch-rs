use super::{PlayerInput, TickNumber, PlayerId, GameInfo};

#[derive(Debug, Clone)]
pub enum Channel {
    Messages,
    Ticks,
} 
pub const NUM_CHANNELS: usize = 2;

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct TimedPlayerInput {
    pub duration_s: f32,
    pub input: PlayerInput,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum ClientMessage {
    Pong,
    WishConnect {
        name: String,
    },
    PlayerInput(TimedPlayerInput),
    StartingTick {
        tick: TickNumber,
    }
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum ServerMessage {
    Ping,
    AcceptConnect {
        your_id: PlayerId,
        game_info: GameInfo,
    },

    // Broadcast messages
    PlayerConnect {
        id: PlayerId,
        name: String,
    },
    PlayerDisconnect {
        id: PlayerId,
    },
}
