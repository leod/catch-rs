use std::fmt;

pub type PlayerId = u32;

#[derive(Clone, CerealData)]
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

impl PlayerInput {
    pub fn new() -> PlayerInput {
        PlayerInput {
            left_pressed: false,
            right_pressed: false,
            forward_pressed: false,
            back_pressed: false,
            use_pressed: false,
        }
    }

    pub fn any(&self) -> bool {
        self.left_pressed ||
        self.right_pressed ||
        self.forward_pressed ||
        self.back_pressed ||
        self.use_pressed
    }
}

impl fmt::Debug for PlayerInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PlayerInput({}{}{}{}{})",
               if self.left_pressed { "A" } else { "" },
               if self.right_pressed { "D" } else { "" },
               if self.forward_pressed { "W" } else { "" },
               if self.back_pressed { "S" } else { "" },
               if self.use_pressed { "X" } else { "" })
    }
}

