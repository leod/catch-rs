use std::collections::HashMap;

use piston::input::{Button, Key, Input};
use piston::input::keyboard::{ModifierKey, NO_MODIFIER, ALT};

pub use shared::player::{PlayerInput, InputKey};

pub struct InputMap {
    inputs: Vec<(ModifierKey, Button, InputKey)>,
}

impl InputMap {
    pub fn new() -> InputMap {
        let inputs = vec![
            (NO_MODIFIER, Button::Keyboard(Key::Left), InputKey::Left),
            (NO_MODIFIER, Button::Keyboard(Key::Right), InputKey::Right),
            (NO_MODIFIER, Button::Keyboard(Key::Up), InputKey::Forward),
            (NO_MODIFIER, Button::Keyboard(Key::Down), InputKey::Back),
            (ALT, Button::Keyboard(Key::Left), InputKey::StrafeLeft),
            (ALT, Button::Keyboard(Key::Right), InputKey::StrafeRight),
            //(NO_MODIFIER, Button::Keyboard(Key::E), InputKey::Use),
            (NO_MODIFIER, Button::Keyboard(Key::LShift), InputKey::Flip),
            (NO_MODIFIER, Button::Keyboard(Key::Space), InputKey::Dash),
        ];

        InputMap {
            inputs: inputs
        }
    }

    pub fn update_player_input(&self,
                               modifier_key: ModifierKey,
                               input: &Input,
                               player_input: &mut PlayerInput) {
        match *input {
            Input::Press(button) =>
                for &(_, ref b, ref input_key) in self.inputs.iter() {
                    if *b == button {
                        player_input.set(*input_key);
                    }
                },
            Input::Release(button) => 
                for &(_, ref b, ref input_key) in self.inputs.iter() {
                    if *b == button {
                        player_input.unset(*input_key);
                    }
                },
            _ => ()
        }

        // Disable any input key that doesn't have its modifiers pressed
        for &(ref modifier, _, ref key) in self.inputs.iter() {
            if modifier.bits() != modifier_key.bits() {
                player_input.unset(*key);
            }
        }
    }
}

