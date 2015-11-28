use ecs::ServiceManager;

use shared::GameEvent;
use shared::services::HasEvents;
use shared::entities::NetEntities;

#[derive(Default)]
pub struct Services {
    pub net_entities: NetEntities,
}

impl ServiceManager for Services {}

impl HasEvents for Services {
    fn add_event(&mut self, _event: &GameEvent) {
        panic!("add_event not implemented yet");
    }
}
