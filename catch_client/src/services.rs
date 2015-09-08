use ecs::ServiceManager;

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
