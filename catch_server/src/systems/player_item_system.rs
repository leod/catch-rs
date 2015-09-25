use ecs;
use ecs::{EntityData, Process, System, DataHelper};

use shared::{ItemSlot, Map, Item, NUM_ITEM_SLOTS};
use shared::net::TimedPlayerInput;
use shared::player::PlayerState;
use shared::player::PlayerInputKey;

use components::Components;
use services::Services;
use entities;

const PROJECTILE_SPEED: f64 = 200.0; 

pub struct PlayerItemSystem;

impl PlayerItemSystem {
    pub fn new() -> PlayerItemSystem {
        PlayerItemSystem
    }

    fn use_item(&self,
                entity: ecs::Entity,
                slot: ItemSlot,
                data: &mut DataHelper<Components, Services>) {
        let (player_id,
             player_position,
             player_orientation,
             item) = data.with_entity_data(&entity, |e, c| {
            (c.net_entity[e].owner,
             c.position[e].p,
             c.orientation[e].angle,
             c.player_state[e].get_item(slot).unwrap().item.clone())
        }).unwrap();

        let new_item = match item {
            Item::Weapon { charges } => {
                let projectile_entity = entities::build_net("bullet", player_id, data);

                data.with_entity_data(&projectile_entity, |projectile_e, c| {
                    c.position[projectile_e].p = player_position;
                    c.orientation[projectile_e].angle = player_orientation;
                    c.linear_velocity[projectile_e].v = [
                        player_orientation.cos() * PROJECTILE_SPEED,
                        player_orientation.sin() * PROJECTILE_SPEED
                    ];
                });

                if charges > 1 {
                    Some(Item::Weapon { charges: charges - 1 })
                } else {
                    None
                }
            },
            item => panic!("item use not implemented: {:?}", item)
        };

        data.with_entity_data(&entity, |e, c| {
            match &new_item {
                &Some(ref item) => {
                    let equipped_item = c.player_state[e].get_item_mut(slot).unwrap();
                    equipped_item.item = item.clone();
                    equipped_item.cooldown_s = item.cooldown_s();
                }
                &None => {
                    c.player_state[e].unequip(slot);
                }
            };
        });
    }

    fn try_use_item(&self,
                    entity: ecs::Entity,
                    slot: ItemSlot,
                    data: &mut DataHelper<Components, Services>) {
        let can_use = data.with_entity_data(&entity, |e, c| {
            match c.player_state[e].get_item(slot) {
                Some(equipped_item) => equipped_item.cooldown_s.is_none(),
                None => false,
            }
        }).unwrap();

        if can_use {
            self.use_item(entity, slot, data);
        }
    }

    pub fn run_player_input(&self,
                            entity: ecs::Entity,
                            timed_input: &TimedPlayerInput,
                            map: &Map,
                            data: &mut DataHelper<Components, Services>) {
        let dur_s = timed_input.duration_s;
        let input = &timed_input.input;

        data.with_entity_data(&entity, |e, c| {
            // Check item cooldowns
            for i in 0..NUM_ITEM_SLOTS {
                if let Some(equipped_item) = c.player_state[e].get_item_mut(i) {
                    if let Some(cooldown_s) = equipped_item.cooldown_s {
                        let cooldown_s = cooldown_s - dur_s;
                        equipped_item.cooldown_s =
                            if cooldown_s <= 0.0 { None }
                            else { Some(cooldown_s) };
                    }
                }
            }

            if input.has(PlayerInputKey::Equip) {
                // Equipping items

                let hidden_item = c.full_player_state[e].hidden_item.clone();
                if let Some(hidden_item) = hidden_item {
                    let slot = if input.has(PlayerInputKey::Item1) {
                        Some(0)
                    } else if input.has(PlayerInputKey::Item2) {
                        Some(1)
                    } else if input.has(PlayerInputKey::Item3) {
                        Some(2)
                    } else {
                        None
                    };

                    if let Some(slot) = slot {
                        debug!("player {} equipping item {:?} to slot {}",
                               c.net_entity[e].owner, hidden_item, slot);

                        c.player_state[e].equip(slot, hidden_item);
                        c.full_player_state[e].hidden_item = None;
                    }
                }
            }
        });

        // Using items
        if !input.has(PlayerInputKey::Equip) {
            let used_slots = data.with_entity_data(&entity, |e, c| {
                let mut used_slots = Vec::new();
                if input.has(PlayerInputKey::Item1) {
                    used_slots.push(0);
                }
                if input.has(PlayerInputKey::Item2) {
                    used_slots.push(1);
                }
                if input.has(PlayerInputKey::Item3) {
                    used_slots.push(2);
                }
                used_slots
            }).unwrap();

            for &slot in used_slots.iter() {
                self.try_use_item(entity, slot, data);
            }
        }
    }
}

impl System for PlayerItemSystem {
    type Components = Components;
    type Services = Services;
}

impl Process for PlayerItemSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
