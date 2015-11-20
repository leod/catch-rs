use std::f32;

use ecs::{EntityData, DataHelper};
use na::{Norm, Vec2};

use shared::{GameEvent, DeathReason, NEUTRAL_PLAYER_ID};
use shared::services::HasEvents;

use entities;
use components::Components;
use services::Services;
use systems::interaction_system::{InteractionResponse, Interaction};

/// Kill player on hitting enemy, or kill enemy if player is dashing
pub struct PlayerBouncyEnemyInteraction;
impl Interaction for PlayerBouncyEnemyInteraction {
    fn condition(&self,
                 player: EntityData<Components>, enemy: EntityData<Components>,
                 data: &mut DataHelper<Components, Services>) -> bool {
        data.net_entity[player].owner != data.net_entity[enemy].owner &&
        (data.player_state[player].vulnerable() ||
         data.player_state[player].dashing.is_some())
    }
    fn apply(&self,
             player: EntityData<Components>, enemy: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) -> InteractionResponse {
        if data.player_state[player].vulnerable() {
            let owner = data.net_entity[player].owner;
            let position = data.position[player].p;
            data.services.add_event(&GameEvent::PlayerDied {
                player_id: owner,
                position: position,
                responsible_player_id: NEUTRAL_PLAYER_ID,
                reason: DeathReason::BouncyBall,
            });
        } else {
            assert!(data.player_state[player].dashing.is_some());

            let event = GameEvent::EnemyDied {
                position: data.position[enemy].p
            };
            data.services.add_event(&event);

            entities::remove_net(**enemy, data);
        }
        InteractionResponse::None
    }
}

/// Bouncy enemies bounce off each other
pub struct BouncyEnemyInteraction;
impl Interaction for BouncyEnemyInteraction {
    fn apply(&self,
             a: EntityData<Components>, b: EntityData<Components>,
             c: &mut DataHelper<Components, Services>) -> InteractionResponse {
        /*let n = (c.linear_velocity[a].v + c.linear_velocity[b].v).normalize();
        let n_angle = n[1].atan2(n[0]);
        c.orientation[a].angle = 2.0 * n_angle - c.orientation[a].angle;
        c.orientation[b].angle = 2.0 * n_angle - c.orientation[b].angle;*/

        let delta = c.position[b].p - c.position[a].p;
        let alpha = delta[1].atan2(delta[0]);

        /*c.orientation[a].angle = alpha;
        c.orientation[b].angle = f32::consts::PI + alpha;*/

        c.orientation[a].angle = f32::consts::PI + alpha;
        c.orientation[b].angle = alpha;

        let direction_a = Vec2::new(c.orientation[a].angle.cos(),
                                    c.orientation[a].angle.sin());
        let direction_b = Vec2::new(c.orientation[b].angle.cos(),
                                    c.orientation[b].angle.sin());
        c.linear_velocity[a].v = direction_a * c.linear_velocity[a].v.norm();
        c.linear_velocity[b].v = direction_b * c.linear_velocity[b].v.norm();
        
        /*c.linear_velocity[b].v = c.linear_velocity[b].v + direction_b;
        c.linear_velocity[a].v = c.linear_velocity[a].v + direction_a;*/

        InteractionResponse::DisplaceNoOverlap
    }
}

/// Picking up items
pub struct PlayerItemInteraction;
impl Interaction for PlayerItemInteraction {
    fn apply(&self,
             player: EntityData<Components>, item: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) -> InteractionResponse {
        data.full_player_state[player].hidden_item = Some(data.item[item].clone());
        entities::remove_net(**item, data);

        let owner = data.net_entity[player].owner;
        let position = data.position[item].p;
        data.services.add_event(&GameEvent::PlayerTakeItem {
           player_id: owner,
           position: position,
        });

        InteractionResponse::None
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
             data:  &mut DataHelper<Components, Services>) -> InteractionResponse {
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

        InteractionResponse::None
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
             data: &mut DataHelper<Components, Services>) -> InteractionResponse {
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

        InteractionResponse::None
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
             data: &mut DataHelper<Components, Services>) -> InteractionResponse {
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

        InteractionResponse::None
    }
}
