use ecs::{BuildData};

use shared;

use components::{Components, DrawPlayer, DrawBouncyEnemy, DrawItem};

/// Adds client-side components that are not synchronized over the net to an entity
pub fn build_client(type_name: &str,
                    entity: BuildData<Components>,
                    data: &mut Components) {
    if type_name == "player" {
        data.draw_player.add(&entity, DrawPlayer::default());
    } else if type_name == "bouncy_enemy" {
        data.draw_bouncy_enemy.add(&entity, DrawBouncyEnemy::default());
    } else if type_name == "item" {
        data.draw_item.add(&entity, DrawItem::default());
    } else if type_name == "item_spawn" {
    } else {
        panic!("Unknown entity type: {}", type_name);
    }
}
