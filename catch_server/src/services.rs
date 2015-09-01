use shared::tick::Tick;

services! {
    struct Services {
        next_tick: Option<Tick> = None
    }
}

