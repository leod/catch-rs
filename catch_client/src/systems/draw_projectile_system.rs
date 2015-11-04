use ecs::{Aspect, System, DataHelper, Process};
use na::{Vec4, Mat2, Mat4};

use shared::util::CachedAspect;

use components::{Components, Shape};
use services::Services;
use draw::{DrawElement, DrawList, DrawAttributes};

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
            let p = data.position[entity].p;
            let alpha = data.orientation[entity].angle;

            let (width, height) = match data.shape[entity] {
                Shape::Rect { width, height } => (width, height),
                _ => panic!("projectile should be rect"),
            };

            let rot_mat = Mat2::new(alpha.cos(), -alpha.sin(),
                                    alpha.sin(), alpha.cos());
            let scale_mat = Mat2::new(width, 0.0,
                                      0.0, height);
            let m = rot_mat * scale_mat;
            let model_mat = Mat4::new(m.m11, m.m12, 0.0, p.x,
                                      m.m21, m.m22, 0.0, p.y,
                                      0.0, 0.0, 1.0, 0.0,
                                      0.0, 0.0, 0.0, 1.0);
            draw_list.push((DrawElement::Square, DrawAttributes {
                color: Vec4::new(0.4, 0.4, 0.4, 1.0),
                model_mat: model_mat,
            }));
        }
    }
}

impl_cached_system!(Components, Services, DrawProjectileSystem, aspect);

impl Process for DrawProjectileSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
