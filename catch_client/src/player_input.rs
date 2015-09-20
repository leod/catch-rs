use std::collections::HashMap;

use piston::input::{Button, Key, Input};
use piston::input::keyboard::{ModifierKey, NO_MODIFIER, ALT};

pub use shared::player::{PlayerInput, InputKey};

pub struct InputMap {
    inputs: Vec<(Button, InputKey)>,
}

impl InputMap {
    pub fn new() -> InputMap {
        let inputs = vec![
            (Button::Keyboard(Key::Left), InputKey::Left),
            (Button::Keyboard(Key::Right), InputKey::Right),
            (Button::Keyboard(Key::Up), InputKey::Forward),
            (Button::Keyboard(Key::Down), InputKey::Back),

            (Button::Keyboard(Key::LAlt), InputKey::Strafe),

            (Button::Keyboard(Key::LShift), InputKey::Flip),
            (Button::Keyboard(Key::Space), InputKey::Dash),

            (Button::Keyboard(Key::Q), InputKey::Item1),
            (Button::Keyboard(Key::W), InputKey::Item2),
            (Button::Keyboard(Key::E), InputKey::Item3),

            (Button::Keyboard(Key::LCtrl), InputKey::Equip),
        ];

        InputMap {
            inputs: inputs
        }
    }

    pub fn update_player_input(&self,
                               input: &Input,
                               player_input: &mut PlayerInput) {
        match *input {
            Input::Press(button) =>
                for &(ref b, ref input_key) in self.inputs.iter() {
                    if *b == button {
                        player_input.set(*input_key);
                    }
                },
            Input::Release(button) => 
                for &(ref b, ref input_key) in self.inputs.iter() {
                    if *b == button {
                        player_input.unset(*input_key);
                    }
                },
            _ => ()
        }
    }
}

