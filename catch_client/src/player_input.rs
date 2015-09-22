use piston::input::{Button, Key, Input};

pub use shared::player::{PlayerInput, PlayerInputKey};

pub struct InputMap {
    inputs: Vec<(Button, PlayerInputKey)>,
}

impl InputMap {
    pub fn new() -> InputMap {
        let inputs = vec![
            (Button::Keyboard(Key::Left), PlayerInputKey::Left),
            (Button::Keyboard(Key::Right), PlayerInputKey::Right),
            (Button::Keyboard(Key::Up), PlayerInputKey::Forward),
            (Button::Keyboard(Key::Down), PlayerInputKey::Back),

            (Button::Keyboard(Key::LAlt), PlayerInputKey::Strafe),

            (Button::Keyboard(Key::LShift), PlayerInputKey::Flip),
            (Button::Keyboard(Key::Space), PlayerInputKey::Dash),

            (Button::Keyboard(Key::Q), PlayerInputKey::Item1),
            (Button::Keyboard(Key::W), PlayerInputKey::Item2),
            (Button::Keyboard(Key::E), PlayerInputKey::Item3),

            (Button::Keyboard(Key::LCtrl), PlayerInputKey::Equip),
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

