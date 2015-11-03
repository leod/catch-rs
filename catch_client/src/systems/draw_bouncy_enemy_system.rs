use na::{Vec4, Mat4};
use ecs::{Aspect, System, DataHelper, Process};

use shared::{NEUTRAL_PLAYER_ID};
use shared::util::CachedAspect;

use components::{Components, Shape};
use services::Services;
use draw::{DrawElement, DrawList, DrawAttributes};

pub struct DrawBouncyEnemySystem {
    aspect: CachedAspect<Components>,
}

impl DrawBouncyEnemySystem {
    pub fn new(aspect: Aspect<Components>) -> DrawBouncyEnemySystem {
        DrawBouncyEnemySystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, draw_list: &mut DrawList) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;

            let r = match data.shape[entity] {
                Shape::Circle { radius } => radius,
                _ => panic!("enemy should be circle"),
            };

            let color = if data.net_entity[entity].owner == NEUTRAL_PLAYER_ID {
                Vec4::new(1.0, 0.0, 0.0, 1.0)
            } else {
                Vec4::new(0.0, 0.0, 1.0, 1.0)
            };
            let model_mat = Mat4::new(r, 0.0, 0.0, p.x,
                                      0.0, r, 0.0, p.y,
                                      0.0, 0.0, 1.0, 0.0,
                                      0.0, 0.0, 0.0, 1.0);
            draw_list.push((DrawElement::Circle, DrawAttributes {
                color: color,
                model_mat: model_mat,
            }));
        }
    }
}

impl_cached_system!(Components, Services, DrawBouncyEnemySystem, aspect);

impl Process for DrawBouncyEnemySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
