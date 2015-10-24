use ecs::ServiceManager;

use shared::GameEvent;
use shared::services::HasEvents;

pub struct Services {
    x: u32 
}

impl Default for Services {
    fn default() -> Services {
        Services {
            x: 0
        }
    }
}

impl ServiceManager for Services {}

impl HasEvents for Services {
    fn add_event(&mut self, event: &GameEvent) {
        panic!("add_event not implemented yet");
    }
}
