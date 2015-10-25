use ecs::ServiceManager;

use shared::GameEvent;
use shared::services::HasEvents;

#[derive(Default)]
pub struct Services {
    _x: u32 
}

impl ServiceManager for Services {}

impl HasEvents for Services {
    fn add_event(&mut self, _event: &GameEvent) {
        panic!("add_event not implemented yet");
    }
}
