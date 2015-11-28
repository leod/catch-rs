use ecs::{Aspect, Process, System, DataHelper, EntityData};
use na::Vec2;

use shared::{ItemSlot, GameEvent, Item, NUM_ITEM_SLOTS};
use shared::movement;
use shared::net::TimedPlayerInput;
use shared::player::PlayerInputKey;
use shared::services::HasEvents;
use shared::util::CachedAspect;

use components::Components;
use services::Services;
use entities;

const PROJECTILE_SPEED: f32 = 200.0; 

/// System for interpreting player input on the server side
pub struct PlayerControllerSystem {
    player_aspect: CachedAspect<Components>,
    wall_aspect: CachedAspect<Components>,
}

impl PlayerControllerSystem {
    pub fn new(player_aspect: Aspect<Components>,
               wall_aspect: Aspect<Components>) -> PlayerControllerSystem {
        PlayerControllerSystem {
            player_aspect: CachedAspect::new(player_aspect),
            wall_aspect: CachedAspect::new(wall_aspect),
        }
    }

    pub fn run_queued_inputs(&self, data: &mut DataHelper<Components, Services>) {
        for player in self.player_aspect.iter() {
            let inputs = data.player_controller[player].inputs.clone();
            data.player_controller[player].inputs.clear();

            let owner = data.net_entity[player].owner;

            for input in &inputs {
                movement::run_player_movement_input(player, owner, input, &self.wall_aspect, data);

                self.run_item_input(input, player, data);
            }
        }
    }

    fn run_item_input(&self, timed_input: &TimedPlayerInput, e: EntityData<Components>,
                      c: &mut DataHelper<Components, Services>) {
        let dur_s = timed_input.duration_s;
        let input = &timed_input.input;

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

                    let player_id = c.net_entity[e].owner;
                    let p = c.position[e].p;
                    c.services.add_event(&GameEvent::PlayerEquipItem {
                        player_id: player_id,
                        position: p,
                        item: hidden_item.clone(),
                    });
                }
            }
        }

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
                self.try_use_item(slot, e, c);
            }
        }
    }

    fn use_item(&self,
                slot: ItemSlot,
                e: EntityData<Components>,
                c: &mut DataHelper<Components, Services>) {
        let player_id = c.net_entity[e].owner;
        let p = c.position[e].p;
        let angle = c.orientation[e].angle;
        let item = c.player_state[e].get_item(slot).unwrap().item.clone();

        let new_item = match item {
            Item::Weapon { charges } => {
                let projectile_entity = entities::build_net("bullet", player_id, c);

                c.with_entity_data(&projectile_entity, |projectile_e, c| {
                    c.position[projectile_e].p = p;
                    c.orientation[projectile_e].angle = angle;
                    c.linear_velocity[projectile_e].v = Vec2::new(
                        angle.cos() * PROJECTILE_SPEED,
                        angle.sin() * PROJECTILE_SPEED
                    );
                });

                if charges > 1 {
                    Some(Item::Weapon { charges: charges - 1 })
                } else {
                    None
                }
            }
            Item::BallSpawner { charges } => {
                let orbit_entity = entities::build_net("bouncy_enemy", player_id, c);

                c.with_entity_data(&orbit_entity, |e_bouncy, c| {
                    c.position[e_bouncy].p = p + Vec2::new(10.0, 0.0);
                    c.bouncy_enemy[e_bouncy].orbit = Some(**e);
                });

                if charges > 1 {
                    Some(Item::BallSpawner { charges: charges - 1 })
                } else {
                    None
                }
            }
            Item::Shield => {
                c.player_state[e].has_shield = true;
                None
            }
            item => panic!("item use not implemented: {:?}", item)
        };

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
    }

    fn try_use_item(&self,
                    slot: ItemSlot,
                    e: EntityData<Components>,
                    c: &mut DataHelper<Components, Services>) {
        let can_use = 
            match c.player_state[e].get_item(slot) {
                Some(equipped_item) => equipped_item.cooldown_s.is_none(),
                None => false,
            };

        if can_use {
            self.use_item(slot, e, c);
        }
    }
}

impl_cached_system!(Components, Services, PlayerControllerSystem, player_aspect, wall_aspect);

impl Process for PlayerControllerSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
