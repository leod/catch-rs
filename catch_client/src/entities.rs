use ecs::{BuildData};

use shared;

use components::{Components, DrawPlayer, DrawBouncyEnemy};

/// Adds client-side components that are not synchronized over the net to an entity
pub fn build(type_name: &str,
             entity: BuildData<Components>,
             data: &mut Components) {
    shared::entities::build(type_name, entity, data);

    if type_name == "player" {
        data.draw_player.add(&entity, DrawPlayer::default());
    } else if type_name == "bouncy_enemy" {
        data.draw_bouncy_enemy.add(&entity, DrawBouncyEnemy::default());
    } else {
        panic!("Unknown entity type: {}", type_name);
    }
}
