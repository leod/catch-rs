use shared::tick::Tick;

services! {
    struct Services {
        cur_tick: Option<Tick> = None
    }
}

