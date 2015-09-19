#[derive(CerealData, Clone)]
pub enum Item {
    Weapon {
        charges: usize,
    },
    SpeedBoost {
        duration_s: f64,
    },
    BlockPlacer {
        charges: usize,
    }
}
