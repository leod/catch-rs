use std::f64;
use ecs::{EntityData, DataHelper};

use shared::math;
use shared::{GameEvent, NEUTRAL_PLAYER_ID};

use entities;
use components::Components;
use services::Services;
use systems::interaction_system::Interaction;

/// Kill player on hitting enemy
pub struct PlayerBouncyEnemyInteraction;
impl Interaction for PlayerBouncyEnemyInteraction {
    fn apply(&self,
             player_e: EntityData<Components>, _enemy_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        if data.player_state[player_e].vulnerable() {
            let owner = data.net_entity[player_e].owner;
            let position = data.position[player_e].p;
            data.services.add_event_to_run(&GameEvent::PlayerDied {
                player_id: owner,
                position: position,
                responsible_player_id: NEUTRAL_PLAYER_ID
            });
        }
    }
}

/// Bouncy enemies bounce off each other
pub struct BouncyEnemyInteraction;
impl Interaction for BouncyEnemyInteraction {
    fn apply(&self,
             a_e: EntityData<Components>, b_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        // Flip orientations of both entities and add some velocity in the new direction

        data.orientation[a_e].angle = data.orientation[a_e].angle + f64::consts::PI;
        let direction_a = [data.orientation[a_e].angle.cos(),
                           data.orientation[a_e].angle.sin()];
        data.linear_velocity[a_e].v = math::add(data.linear_velocity[a_e].v,
                                                math::scale(direction_a, 500.0));

        data.orientation[b_e].angle = data.orientation[b_e].angle + f64::consts::PI;
        let direction_b = [data.orientation[b_e].angle.cos(),
                           data.orientation[b_e].angle.sin()];
        data.linear_velocity[b_e].v = math::add(data.linear_velocity[b_e].v,
                                                math::scale(direction_b, 500.0));
    }
}

/// Picking up items
pub struct PlayerItemInteraction;
impl Interaction for PlayerItemInteraction {
    fn apply(&self,
             player_e: EntityData<Components>, item_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        data.full_player_state[player_e].hidden_item = Some(data.item[item_e].clone());
        entities::remove_net(**item_e, data);

        let owner = data.net_entity[player_e].owner;
        let position = data.position[player_e].p;
        data.services.add_event(&GameEvent::PlayerTakeItem {
           player_id: owner,
           position: position,
        });
    }
}

/// Projectiles kill enemies
pub struct ProjectileBouncyEnemyInteraction;
impl Interaction for ProjectileBouncyEnemyInteraction {
    fn condition(&self,
                 projectile_e: EntityData<Components>, enemy_e: EntityData<Components>,
                 data: &mut DataHelper<Components, Services>) -> bool {
        data.net_entity[projectile_e].owner != data.net_entity[enemy_e].owner
    }

    fn apply(&self,
             projectile_e: EntityData<Components>, enemy_e: EntityData<Components>,
             data:  &mut DataHelper<Components, Services>) {
        let position = data.position[enemy_e].p;
        data.services.add_event(&GameEvent::EnemyDied {
            position: position,
        });

        let position = data.position[projectile_e].p;
        data.services.add_event(&GameEvent::ProjectileImpact {
            position: position,
        });

        entities::remove_net(**projectile_e, data);
        entities::remove_net(**enemy_e, data);
    }
}

/// Projectiles kill players (for now; there will be other types of projectiles too)
pub struct ProjectilePlayerInteraction;
impl Interaction for ProjectilePlayerInteraction {
    fn condition(&self,
                 projectile_e: EntityData<Components>, player_e: EntityData<Components>,
                 data: &mut DataHelper<Components, Services>) -> bool {
        data.net_entity[projectile_e].owner != data.net_entity[player_e].owner
    }

    fn apply(&self,
             projectile_e: EntityData<Components>, player_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        let player_id = data.net_entity[player_e].owner;
        let position = data.position[player_e].p;
        let responsible_player_id = data.net_entity[projectile_e].owner;
        data.services.add_event_to_run(&GameEvent::PlayerDied {
            player_id: player_id,
            position: position,
            responsible_player_id: responsible_player_id,
        });

        let position = data.position[projectile_e].p;
        data.services.add_event(&GameEvent::ProjectileImpact {
            position: position,
        });

        entities::remove_net(**projectile_e, data); 
    }
}

/// Players catch each other
pub struct PlayerPlayerInteraction;
impl Interaction for PlayerPlayerInteraction {
    fn condition(&self,
                 player1_e: EntityData<Components>, player2_e: EntityData<Components>,
                 data: &mut DataHelper<Components, Services>) -> bool {
        data.player_state[player1_e].is_catcher ||
        data.player_state[player2_e].is_catcher
    }

    fn apply(&self,
             player1_e: EntityData<Components>, player2_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        let (catcher_e, catchee_e) = if data.player_state[player1_e].is_catcher {
            (player1_e, player2_e)
        } else {
            (player2_e, player1_e)
        };

        assert!(data.player_state[catcher_e].is_catcher);
        assert!(!data.player_state[catchee_e].is_catcher);
        
        let player_id = data.net_entity[catchee_e].owner;
        let position = data.position[catchee_e].p;
        let responsible_player_id = data.net_entity[catcher_e].owner;

        data.services.add_event_to_run(&GameEvent::PlayerDied {
            player_id: player_id,
            position: position,
            responsible_player_id: responsible_player_id,
        });
    }
}
