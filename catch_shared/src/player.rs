use std::fmt;
use std::io::{Read, Write};

use cereal::{CerealData, CerealResult};

pub type PlayerId = u32;
pub type PlayerInputNumber = u32;

pub const NEUTRAL_PLAYER_ID: PlayerId = 0;

// Component attached to any player for both client and server
#[derive(CerealData, Clone)]
pub struct PlayerState { 
    pub color: u32,
    pub dashing: Option<f64>,

    // States like stunned etc.
}

// Attached to players on the server and the clients controlling them
#[derive(CerealData, Clone)]
pub struct FullPlayerState {
    unchi: bool

    // Item states, cooldowns etc.
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InputKey {
    Left,
    Right,
    Forward,
    Back,

    Strafe,

    Use,
    Flip,
    Dash,

    //Max,
}

pub const NUM_INPUT_KEYS: usize = 9; //usize = InputKey::Max as usize;

#[derive(Clone)]
pub struct PlayerInput {
    pub pressed: [bool; NUM_INPUT_KEYS]
}

impl PlayerInput {
    pub fn has(&self, key: InputKey) -> bool {
        self.pressed[key as usize]
    }

    pub fn set(&mut self, key: InputKey) {
        self.pressed[key as usize] = true;
    }

    pub fn unset(&mut self, key: InputKey) {
        self.pressed[key as usize] = false;
    }
}

#[derive(Debug, Clone, CerealData)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub name: String,
    pub score: i32,
    pub ping_ms: Option<u32>,
    pub alive: bool,
}

impl PlayerInfo {
    pub fn new(id: PlayerId, name: String) -> PlayerInfo {
        PlayerInfo {
            id: id,
            name: name,
            score: 0,
            ping_ms: None,
            alive: false
        }
    }
}

impl PlayerInput {
    pub fn new() -> PlayerInput {
        PlayerInput {
            pressed: [false; NUM_INPUT_KEYS]
        }
    }

    pub fn any(&self) -> bool {
        for i in 0..NUM_INPUT_KEYS {
            if self.pressed[i] {
                return true;
            }
        }
        false
    }
}

impl fmt::Debug for PlayerInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PlayerInput")
    }
}

impl CerealData for PlayerInput {
    fn read(r: &mut Read) -> CerealResult<PlayerInput> {
        // TODO: Use the bits...

        let mut input = [false; NUM_INPUT_KEYS];

        for i in 0..NUM_INPUT_KEYS {
            input[i] = try!(bool::read(r));
        }

        Ok(PlayerInput { pressed: input })
    }

    fn write(&self, w: &mut Write) -> CerealResult<()> {
        for i in 0..NUM_INPUT_KEYS {
            try!(self.pressed[i].write(w));
        }

        Ok(())
    }
}

