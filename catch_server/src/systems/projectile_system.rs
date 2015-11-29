use std::f32;

use hprof;
use rand;
use ecs::{EntityData, Aspect, Process, System, DataHelper};
use na::{Vec2, Norm};

use shared::GameEvent;
use shared::util::CachedAspect;
use shared::services::HasEvents;

use components::{Components, Projectile, Shape};
use services::Services;
use entities;

pub struct ProjectileSystem {
    aspect: CachedAspect<Components>,
}

pub fn explode(projectile: EntityData<Components>, data: &mut DataHelper<Components, Services>) {
    const NUM_SHRAPNELS: usize = 15;

    let strength = match data.projectile[projectile] {
        Projectile::Frag(_) => {
            for _ in 0..NUM_SHRAPNELS {
                let player_id = data.net_entity[projectile].owner;
                let angle = rand::random::<f32>() * f32::consts::PI * 2.0;
                //let angular_velocity = rand::random::<f32>() * f32::consts::PI * 5.0;
                let speed = 50.0 + 100.0 * rand::random::<f32>();
                let linear_velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                let width = rand::random::<f32>() * 6.0 + 3.0;
                let height = rand::random::<f32>() * 6.0 + 3.0;
                let shape = Shape::Rect { width: width, height: height };

                let shrapnel = entities::build_net("shrapnel", player_id, data);
                data.with_entity_data(&shrapnel, |shrapnel, c| {
                    c.position[shrapnel].p = c.position[projectile].p;
                    c.orientation[shrapnel].angle = angle;
                    c.angular_velocity[shrapnel].v = 0.0;
                    c.linear_velocity[shrapnel].v = linear_velocity;
                    c.shape[shrapnel] = shape.clone();
                });
            }
            4.0
        }
        Projectile::Shrapnel => 0.9,
        Projectile::Bullet => 1.0,
    };

    let event = GameEvent::ProjectileImpact {
        position: data.position[projectile].p,
        strength: strength,
    };
    data.services.add_event(&event);
    entities::remove_net(**projectile, data); 
}

impl ProjectileSystem {
    pub fn new(aspect: Aspect<Components>) -> ProjectileSystem {
        ProjectileSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn tick(&self, data: &mut DataHelper<Components, Services>) {
        let _g = hprof::enter("projectile");

        let dur_s = data.services.tick_dur_s;

        for e in self.aspect.iter() {
            match data.projectile[e].clone() {
                Projectile::Frag(lifetime_s) => {
                    if lifetime_s < dur_s {
                        explode(e, data);    
                    } else {
                        data.projectile[e] = Projectile::Frag(lifetime_s - dur_s);
                        data.linear_velocity[e].v = data.linear_velocity[e].v -
                                                    data.linear_velocity[e].v * 1.0 * dur_s;
                        data.angular_velocity[e].v -= data.angular_velocity[e].v * 1.0 * dur_s;
                        if data.angular_velocity[e].v <= 0.0 {
                            data.angular_velocity[e].v = 0.0;
                        }
                    }
                }
                Projectile::Shrapnel => {
                    data.linear_velocity[e].v = data.linear_velocity[e].v -
                                                data.linear_velocity[e].v * 0.5 * dur_s;
                    if data.linear_velocity[e].v.norm() <= 60.0 {
                        explode(e, data);
                    }
                }
                _ => {}
            };
        }
    }
}

impl_cached_system!(Components, Services, ProjectileSystem, aspect);

impl Process for ProjectileSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
