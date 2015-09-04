pub use shared::components::{Position, Orientation, PlayerState};
pub use shared::net::{NetEntity};

components! {
    struct Components {
        #[hot] position: Position,
        #[hot] orientation: Orientation,
        #[hot] net_entity: NetEntity,

        #[cold] player_state: PlayerState,
    }
}
