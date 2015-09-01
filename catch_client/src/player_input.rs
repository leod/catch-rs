use shared::player::PlayerInput;
use piston::input::{Button, Key};

pub struct InputMap {
    pub left_button: Button,
    pub right_button: Button,
    pub forward_button: Button,
    pub back_button: Button,
    pub use_button: Button
}

impl InputMap {
    pub fn update_player_input(&self, input: &piston::Input, player_input: &mut PlayerInput) {
        match input {
            Input::Press(button) => {
                if input == input.left_button {
                    player_input.left_pressed = true;
                } else if input == input.right_button {
                    player_input.right_pressed = true;
                } else if input == input.forward_button {
                    player_input.forward_pressed = true;
                } else if input == input.back_button {
                    player_input.back_pressed = true;
                } else if input == input.use_button {
                    player_input.use_pressed = true;
                }
            }
            Input::Release(button) => {
                if input == input.left_button {
                    player_input.left_pressed = false;
                } else if input == input.right_button {
                    player_input.right_pressed = false;
                } else if input == input.forward_button {
                    player_input.forward_pressed = false;
                } else if input == input.back_button {
                    player_input.back_pressed = false;
                } else if input == input.use_button {
                    player_input.use_pressed = false;
                }
            }
        }
    }
}

