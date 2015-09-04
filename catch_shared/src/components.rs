use math;

#[derive(CerealData, Clone)]
pub struct Position {
    pub p: math::Vec2,
}

#[derive(CerealData, Clone)]
pub struct Orientation {
    pub angle: f64,
}

#[derive(CerealData, Clone)]
pub struct PlayerState {
    pub color: u32
}

impl Default for Position {
    fn default() -> Position {
        Position {
            p: [0.0, 0.0]
        }
    }
}

impl Default for Orientation {
    fn default() -> Orientation {
        Orientation {
            angle: 0.0
        }
    }
}

impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState {
            color: 0,
        }
    }
}
