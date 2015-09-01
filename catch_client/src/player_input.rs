use shared::player::PlayerInput;
use piston::input::{Button, Key, Input};

pub struct InputMap {
    pub left_button: Button,
    pub right_button: Button,
    pub forward_button: Button,
    pub back_button: Button,
    pub use_button: Button
}

impl InputMap {
    pub fn new() -> InputMap {
        InputMap {
            left_button: Button::Keyboard(Key::A),
            right_button: Button::Keyboard(Key::D),
            forward_button: Button::Keyboard(Key::W),
            back_button: Button::Keyboard(Key::S),
            use_button: Button::Keyboard(Key::Space),
        }
    }

    pub fn update_player_input(&self, input: &Input, player_input: &mut PlayerInput) {
        match *input {
            Input::Press(button) => {
                if button == self.left_button {
                    player_input.left_pressed = true;
                } else if button == self.right_button {
                    player_input.right_pressed = true;
                } else if button == self.forward_button {
                    player_input.forward_pressed = true;
                } else if button == self.back_button {
                    player_input.back_pressed = true;
                } else if button == self.use_button {
                    player_input.use_pressed = true;
                }
            }
            Input::Release(button) => {
                if button == self.left_button {
                    player_input.left_pressed = false;
                } else if button == self.right_button {
                    player_input.right_pressed = false;
                } else if button == self.forward_button {
                    player_input.forward_pressed = false;
                } else if button == self.back_button {
                    player_input.back_pressed = false;
                } else if button == self.use_button {
                    player_input.use_pressed = false;
                }
            }
            _ => ()
        }
    }
}

