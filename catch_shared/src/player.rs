pub type PlayerId = u32;

#[derive(Debug, Clone, CerealData)]
pub struct PlayerInput {
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub forward_pressed: bool,
    pub back_pressed: bool,
    pub use_pressed: bool
}

#[derive(Debug, Clone, CerealData)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub name: String,
    pub score: i32,
    pub ping_ms: Option<u32>,
}

impl PlayerInfo {
    pub fn new(id: PlayerId, name: String) -> PlayerInfo {
        PlayerInfo {
            id: id,
            name: name,
            score: 0,
            ping_ms: None
        }
    }
}
