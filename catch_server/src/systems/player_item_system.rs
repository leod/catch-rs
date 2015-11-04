use ecs;
use ecs::{Process, System, DataHelper};
use na::{Vec2, Norm};

use shared::{ItemSlot, GameEvent, Item, NUM_ITEM_SLOTS};
use shared::net::TimedPlayerInput;
use shared::player::PlayerInputKey;
use shared::services::HasEvents;

use components::Components;
use services::Services;
use entities;

pub struct PlayerItemSystem;

impl PlayerItemSystem {
    pub fn new() -> PlayerItemSystem {
        PlayerItemSystem
    }

    pub fn run_player_input(&self,
                            entity: ecs::Entity,
                            timed_input: &TimedPlayerInput,
                            data: &mut DataHelper<Components, Services>) {
        let dur_s = timed_input.duration_s;
        let input = &timed_input.input;

        let mut events = Vec::new();

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

                        c.player_state[e].equip(slot, hidden_item.clone());
                        c.full_player_state[e].hidden_item = None;

                        events.push(GameEvent::PlayerEquipItem {
                            player_id: c.net_entity[e].owner,
                            position: c.position[e].p,
                            item: hidden_item.clone(),
                        });
                    }
                }
            }
        });

        // Using items
        if !input.has(PlayerInputKey::Equip) {
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

            for &slot in used_slots.iter() {
                self.try_use_item(entity, slot, data);
            }
        }

        for event in events.iter() {
            data.services.add_event(&event.clone());
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
