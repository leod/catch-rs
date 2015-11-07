use std::f32;

use ecs::{EntityData, DataHelper};
use na::Vec2;

use shared::{GameEvent, DeathReason, NEUTRAL_PLAYER_ID};
use shared::services::HasEvents;

use entities;
use components::Components;
use services::Services;
use systems::interaction_system::Interaction;

/// Kill player on hitting enemy
pub struct PlayerBouncyEnemyInteraction;
impl Interaction for PlayerBouncyEnemyInteraction {
    fn condition(&self,
                 player: EntityData<Components>, enemy: EntityData<Components>,
                 data: &mut DataHelper<Components, Services>) -> bool {
        data.net_entity[player].owner != data.net_entity[enemy].owner &&
        data.player_state[player].vulnerable()
    }
    fn apply(&self,
             player: EntityData<Components>, _enemy: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        let owner = data.net_entity[player].owner;
        let position = data.position[player].p;
        data.services.add_event(&GameEvent::PlayerDied {
            player_id: owner,
            position: position,
            responsible_player_id: NEUTRAL_PLAYER_ID,
            reason: DeathReason::BouncyBall,
        });
    }
}

/// Bouncy enemies bounce off each other
pub struct BouncyEnemyInteraction;
impl Interaction for BouncyEnemyInteraction {
    fn apply(&self,
             a: EntityData<Components>, b: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        // Flip orientations of both entities and add some velocity in the new direction
                /*data.orientation[e].angle = f32::consts::PI + n_angle - (angle - n_angle);
                data.server_net_entity[e].force(ComponentType::Orientation);

                let v = data.linear_velocity[e].v;
                let speed = math::square_len(v).sqrt();
                data.linear_velocity[e].v = [
                    data.orientation[e].angle.cos() * (speed + 1.0),
                    data.orientation[e].angle.sin() * (speed + 1.0),
                ];*/

        data.orientation[a].angle = data.orientation[a].angle + f32::consts::PI / 2.0;
        let direction_a = Vec2::new(data.orientation[a].angle.cos(),
                                    data.orientation[a].angle.sin());
        data.linear_velocity[a].v = data.linear_velocity[a].v + direction_a * 200.0;

        data.orientation[b].angle = data.orientation[b].angle + f32::consts::PI / 2.0;
        let direction_b = Vec2::new(data.orientation[b].angle.cos(),
                                    data.orientation[b].angle.sin());
        data.linear_velocity[b].v = data.linear_velocity[b].v + direction_b * 200.0;
    }
}

/// Picking up items
pub struct PlayerItemInteraction;
impl Interaction for PlayerItemInteraction {
    fn apply(&self,
             player: EntityData<Components>, item: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        data.full_player_state[player].hidden_item = Some(data.item[item].clone());
        entities::remove_net(**item, data);

        let owner = data.net_entity[player].owner;
        let position = data.position[item].p;
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
                 projectile: EntityData<Components>, enemy: EntityData<Components>,
                 data: &mut DataHelper<Components, Services>) -> bool {
        data.net_entity[projectile].owner != data.net_entity[enemy].owner
    }

    fn apply(&self,
             projectile: EntityData<Components>, enemy: EntityData<Components>,
             data:  &mut DataHelper<Components, Services>) {
        let position = data.position[enemy].p;
        data.services.add_event(&GameEvent::EnemyDied {
            position: position,
        });

        let position = data.position[projectile].p;
        data.services.add_event(&GameEvent::ProjectileImpact {
            position: position,
        });

        entities::remove_net(**projectile, data);
        entities::remove_net(**enemy, data);
    }
}

/// Projectiles kill players (for now; there will be other types of projectiles too)
pub struct ProjectilePlayerInteraction;
impl Interaction for ProjectilePlayerInteraction {
    fn condition(&self,
                 projectile: EntityData<Components>, player: EntityData<Components>,
                 data: &mut DataHelper<Components, Services>) -> bool {
        data.net_entity[projectile].owner != data.net_entity[player].owner &&
        data.player_state[player].vulnerable()
    }

    fn apply(&self,
             projectile: EntityData<Components>, player: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        let player_id = data.net_entity[player].owner;
        let position = data.position[player].p;
        let responsible_player_id = data.net_entity[projectile].owner;
        data.services.add_event(&GameEvent::PlayerDied {
            player_id: player_id,
            position: position,
            responsible_player_id: responsible_player_id,
            reason: DeathReason::Projectile,
        });

        let position = data.position[projectile].p;
        data.services.add_event(&GameEvent::ProjectileImpact {
            position: position,
        });

        entities::remove_net(**projectile, data); 
    }
}

/// Players catch each other
pub struct PlayerPlayerInteraction;
impl Interaction for PlayerPlayerInteraction {
    fn condition(&self,
                 player1: EntityData<Components>, player2: EntityData<Components>,
                 data: &mut DataHelper<Components, Services>) -> bool {
        (data.player_state[player1].is_catcher && data.player_state[player2].vulnerable()) ||
        (data.player_state[player2].is_catcher && data.player_state[player1].vulnerable())
    }

    fn apply(&self,
             player1: EntityData<Components>, player2: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        let (catcher, catchee) = if data.player_state[player1].is_catcher {
            (player1, player2)
        } else {
            (player2, player1)
        };

        assert!(data.player_state[catcher].is_catcher);
        assert!(!data.player_state[catchee].is_catcher);
        
        let player_id = data.net_entity[catchee].owner;
        let position = data.position[catchee].p;
        let responsible_player_id = data.net_entity[catcher].owner;
        data.services.add_event(&GameEvent::PlayerDied {
            player_id: player_id,
            position: position,
            responsible_player_id: responsible_player_id,
            reason: DeathReason::Caught,
        });
    }
}
