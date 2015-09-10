use std::collections::HashMap;

pub use shared::player::{PlayerInput, InputKey};
use piston::input::{Button, Key, Input};

pub struct InputMap {
    map: HashMap<Button, InputKey>
}

impl InputMap {
    pub fn new() -> InputMap {
        let mut map = HashMap::new();

        map.insert(Button::Keyboard(Key::Left), InputKey::Left);
        map.insert(Button::Keyboard(Key::Right), InputKey::Right);
        map.insert(Button::Keyboard(Key::Up), InputKey::Forward);
        map.insert(Button::Keyboard(Key::Down), InputKey::Back);
        map.insert(Button::Keyboard(Key::LAlt), InputKey::Strafe);
        map.insert(Button::Keyboard(Key::E), InputKey::Use);
        map.insert(Button::Keyboard(Key::LShift), InputKey::Flip);
        map.insert(Button::Keyboard(Key::Space), InputKey::Dash);

        InputMap {
            map: map
        }
    }

    pub fn update_player_input(&self, input: &Input, player_input: &mut PlayerInput) {
        match *input {
            Input::Press(button) =>
                match self.map.get(&button) {
                    Some(input_key) =>
                        player_input.set(*input_key),
                    _ =>
                        ()
                },
            Input::Release(button) =>
                match self.map.get(&button) {
                    Some(input_key) =>
                        player_input.unset(*input_key),
                    _ =>
                        ()
                },
            _ => ()
        }
    }
}

