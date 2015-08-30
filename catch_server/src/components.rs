pub use shared::components::{Position};
pub use shared::net::{NetEntity};

components! {
    struct Components {
        #[hot] position: Position,
        #[hot] net_entity: NetEntity
    }
}
