use glium::glutin::{ElementState, VirtualKeyCode};

pub use shared::player::{PlayerInput, PlayerInputKey};

pub struct InputMap {
    inputs: Vec<(VirtualKeyCode, PlayerInputKey)>,
}

impl InputMap {
    pub fn new() -> InputMap {
        let inputs = vec![
            (VirtualKeyCode::Left, PlayerInputKey::Left),
            (VirtualKeyCode::Right, PlayerInputKey::Right),
            (VirtualKeyCode::Up, PlayerInputKey::Forward),
            (VirtualKeyCode::Down, PlayerInputKey::Back),

            (VirtualKeyCode::A, PlayerInputKey::StrafeLeft),
            (VirtualKeyCode::D, PlayerInputKey::StrafeRight),

            (VirtualKeyCode::LShift, PlayerInputKey::Flip),
            (VirtualKeyCode::Space, PlayerInputKey::Dash),

            (VirtualKeyCode::Q, PlayerInputKey::Item1),
            (VirtualKeyCode::W, PlayerInputKey::Item2),
            (VirtualKeyCode::E, PlayerInputKey::Item3),

            (VirtualKeyCode::LControl, PlayerInputKey::Equip),
        ];

        InputMap {
            inputs: inputs
        }
    }

    pub fn update_player_input(&self,
                               state: ElementState, key: VirtualKeyCode,
                               player_input: &mut PlayerInput) {
        match state {
            ElementState::Pressed => {
                for &(k, input_key) in self.inputs.iter() {
                    if k == key {
                        player_input.set(input_key);
                    }
                }
            }
            ElementState::Released => {
                for &(k, input_key) in self.inputs.iter() {
                    if k == key {
                        player_input.unset(input_key);
                    }
                }
            }
        }
    }
}

