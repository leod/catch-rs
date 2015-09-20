use std::fmt;
use std::io::{Read, Write};

use cereal::{CerealData, CerealResult};

pub type PlayerId = u32;
pub type PlayerInputNumber = u32;
pub type ItemSlot = u32;

pub const NEUTRAL_PLAYER_ID: PlayerId = 0;
pub const NUM_ITEM_SLOTS: ItemSlot = 3;

#[derive(Clone, CerealData)]
pub enum Item {
    Weapon {
        charges: usize,
    },
    SpeedBoost {
        duration_s: f64,
    },
    BlockPlacer {
        charges: usize,
    }
}

// Component attached to any player for both client and server
#[derive(CerealData, Clone, Default)]
pub struct PlayerState { 
    pub color: u32,
    pub dashing: Option<f64>,
    pub invulnerable_s: Option<f64>,

    // Equipped items
    pub items: Vec<Option<Item>>,

    // States like stunned etc.
}

impl PlayerState {
    pub fn vulnerable(&self) -> bool {
        self.dashing.is_none() && self.invulnerable_s.is_none()
    }

    pub fn get_item(&self, slot: ItemSlot) -> Option<&Item> {
        assert!(slot < NUM_ITEM_SLOTS);

        if (slot as usize) < self.items.len() {
            self.items[slot as usize].as_ref()
        } else {
            None
        }
    }

    pub fn get_item_mut(&mut self, slot: ItemSlot) -> Option<&mut Item> {
        assert!(slot < NUM_ITEM_SLOTS);

        if (slot as usize) < self.items.len() {
            self.items[slot as usize].as_mut()
        } else {
            None
        }
    }

    pub fn equip(&mut self, slot: ItemSlot, item: Item) {
        if slot as usize >= self.items.len() {
            //self.items.resize(slot as usize+1, None);
            for i in self.items.len()..slot as usize +1 {
                self.items.push(None);
            }

            assert!(self.items.len() == (slot as usize) + 1);
        }

        self.items[slot as usize] = Some(item);
    }
}

// Attached to players on the server and the clients controlling them
// Item states, cooldowns etc.
#[derive(Clone, Default, CerealData)]
pub struct FullPlayerState {
    pub dash_cooldown_s: Option<f64>,

    // An item that the player picked up but hasn't equipped
    pub hidden_item: Option<Item>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InputKey {
    Left,
    Right,
    Forward,
    Back,

    Strafe,

    Flip,
    Dash,

    Item1,
    Item2,
    Item3,

    Equip,

    //Max,
}

pub const NUM_INPUT_KEYS: usize = 11; //usize = InputKey::Max as usize;

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

