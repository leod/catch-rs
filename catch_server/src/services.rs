use ecs::ServiceManager;

use shared::tick::Tick;

//services! {
    pub struct Services {
        pub next_tick: Option<Tick> //= None
    }

    impl Default for Services {
        fn default() -> Services {
            Services {
                next_tick: None,
            }
        }
    }

    impl ServiceManager for Services {}
//}

