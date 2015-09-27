use std::fmt;
use std::io::{Read, Write};

use cereal::{CerealData, CerealResult};

use super::{PlayerId, ItemSlot, NUM_ITEM_SLOTS};

#[derive(Clone, Debug, CerealData)]
pub enum Item {
    Weapon {
        charges: usize,
    },
    SpeedBoost {
        duration_s: f64,
    },
    BlockPlacer {
        charges: usize,
    },
    BallSpawner {
        charges: usize,
    },
}

impl Item {
    pub fn cooldown_s(&self) -> Option<f64> {
        match *self {
            Item::Weapon { charges: _ } => Some(0.7),
            Item::SpeedBoost { duration_s: _ } => None,
            Item::BlockPlacer { charges: _ } => Some(5.0),
            Item::BallSpawner { charges: _ } => Some(2.5),
        }
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

#[derive(Clone, CerealData)]
pub struct EquippedItem {
    pub item: Item,
    pub cooldown_s: Option<f64>, // Some items have a cooldown
}

impl EquippedItem {
    pub fn new(item: Item) -> EquippedItem {
        EquippedItem {
            item: item,
            cooldown_s: None,
        }
    }
}

// Component attached to any player for both client and server
#[derive(Clone, Default, CerealData)]
pub struct PlayerState { 
    pub color: u32,
    pub dashing: Option<f64>,
    pub invulnerable_s: Option<f64>,

    // Equipped items
    pub items: Vec<Option<EquippedItem>>,

    pub is_catcher: bool,
}

impl PlayerState {
    pub fn vulnerable(&self) -> bool {
        self.dashing.is_none() && self.invulnerable_s.is_none()
    }

    pub fn get_item(&self, slot: ItemSlot) -> Option<&EquippedItem> {
        assert!(slot < NUM_ITEM_SLOTS);

        if (slot as usize) < self.items.len() {
            self.items[slot as usize].as_ref()
        } else {
            None
        }
    }

    pub fn get_item_mut(&mut self, slot: ItemSlot) -> Option<&mut EquippedItem> {
        assert!(slot < NUM_ITEM_SLOTS);

        if (slot as usize) < self.items.len() {
            self.items[slot as usize].as_mut()
        } else {
            None
        }
    }

    pub fn equip(&mut self, slot: ItemSlot, item: Item) {
        assert!(slot < NUM_ITEM_SLOTS);

        if slot as usize >= self.items.len() {
            //self.items.resize(slot as usize+1, None);
            for _ in self.items.len()..slot as usize + 1 {
                self.items.push(None);
            }

            assert!(self.items.len() == (slot as usize) + 1);
        }

        self.items[slot as usize] = Some(EquippedItem::new(item));
    }

    pub fn unequip(&mut self, slot: ItemSlot) {
        self.items[slot as usize] = None;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PlayerInputKey {
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
    pub fn has(&self, key: PlayerInputKey) -> bool {
        self.pressed[key as usize]
    }

    pub fn set(&mut self, key: PlayerInputKey) {
        self.pressed[key as usize] = true;
    }

    pub fn unset(&mut self, key: PlayerInputKey) {
        self.pressed[key as usize] = false;
    }
}

#[derive(Debug, Clone, CerealData)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub name: String,
    pub score: i32,
    pub ping_ms: Option<u32>,
}

impl PlayerInfo {
    pub fn new(id: PlayerId, name: String) -> PlayerInfo {
        PlayerInfo {
            id: id,
            name: name,
            score: 0,
            ping_ms: None,
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

