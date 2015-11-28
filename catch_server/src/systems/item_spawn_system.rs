use rand;
use hprof;
use ecs::{Aspect, Process, System, BuildData, DataHelper};

use shared::Item;
use shared::util::CachedAspect;

use components::{Components};
use services::Services;
use entities;

const COOLDOWN_S: f32 = 5.0;

pub struct ItemSpawnSystem {
    aspect: CachedAspect<Components>,
}

impl ItemSpawnSystem {
    pub fn new(aspect: Aspect<Components>) -> ItemSpawnSystem {
        ItemSpawnSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn tick(&self, data: &mut DataHelper<Components, Services>) {
        let _g = hprof::enter("item spawn");

        // Iterate all item spawn entities
        for e in self.aspect.iter() {
            // Did our spawned entity die?
            let spawned_entity_died =
                match data.item_spawn[e].spawned_entity {
                    Some(entity) => 
                        data.with_entity_data(&entity, |_, _| ()).is_none(),
                    None =>
                        false
                };
            if spawned_entity_died {
                assert!(data.item_spawn[e].cooldown_s.is_none());
                data.item_spawn[e].spawned_entity = None; 
                data.item_spawn[e].cooldown_s = Some(COOLDOWN_S);
            }

            // Check cooldown
            let have_cooldown = match data.item_spawn[e].cooldown_s {
                Some(cooldown_s) => {
                    let cooldown_s = cooldown_s - data.services.tick_dur_s;
                    if cooldown_s <= 0.0 { 
                        data.item_spawn[e].cooldown_s = None;
                        false
                    } else {
                        data.item_spawn[e].cooldown_s = Some(cooldown_s);
                        true
                    }
                }
                None => false
            };
            
            // Should we spawn a new item?
            if data.item_spawn[e].spawned_entity.is_none() && !have_cooldown {
                let item_entity = entities::build_net_custom("item", 0, data,
                    |item_e: BuildData<Components>, c: &mut Components| {
                        let choices = vec![
                                           Item::Weapon { charges: 10 },
                                           Item::BallSpawner { charges: 3 },
                                           Item::Shield,
                                          ];
                        let item = choices[rand::random::<usize>() % choices.len()].clone();

                        c.item.add(&item_e, item);
                    });

                data.with_entity_data(&item_entity, |item_e, c| {
                    // Spawn at our position
                    c.position[item_e].p = c.position[e].p;
                });

                data.item_spawn[e].spawned_entity = Some(item_entity);
            }
        }
    }
}

impl_cached_system!(Components, Services, ItemSpawnSystem, aspect);

impl Process for ItemSpawnSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
