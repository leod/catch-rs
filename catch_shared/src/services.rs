use super::GameEvent;

pub trait HasEvents {
    fn add_event(&mut self, event: &GameEvent);
}
