use ecs::{Aspect, System, DataHelper, Process};
use na::Vec4;

use shared::util::CachedAspect;

use components::{Components, Projectile, Shape};
use services::Services;
use draw::DrawList;

pub struct DrawProjectileSystem {
    aspect: CachedAspect<Components>,
}

impl DrawProjectileSystem {
    pub fn new(aspect: Aspect<Components>) -> DrawProjectileSystem {
        DrawProjectileSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, draw_list: &mut DrawList) {
        for entity in self.aspect.iter() {
            let (width, height) = match data.shape[entity] {
                Shape::Rect { width, height } => (width, height),
                _ => panic!("projectile should be rect"),
            };
            let angle = data.orientation[entity].angle;
            let p = data.position[entity].p;

            match data.projectile[entity] {
                Projectile::Frag(_) => {
                    
                }
                _ => {}
            }

            draw_list.push_rect(data.draw_projectile[entity].color, width, height,
                                p, 1.0, angle);
        }
    }
}

impl_cached_system!(Components, Services, DrawProjectileSystem, aspect);

impl Process for DrawProjectileSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
