use std::f32;

use ecs::{Aspect, System, DataHelper, Process};
use na::{Vec2, Vec4, Mat2, Mat4, Norm};

use shared::movement;
use shared::util::CachedAspect;

use components::Components;
use services::Services;
use draw::{DrawElement, DrawList, DrawAttributes};

pub struct DrawWallSystem {
    aspect: CachedAspect<Components>,
}

impl DrawWallSystem {
    pub fn new(aspect: Aspect<Components>) -> DrawWallSystem {
        DrawWallSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, draw_list: &mut DrawList) {
        for entity in self.aspect.iter() {
            let p = data.wall_position[entity].clone();
            let w = Vec2::new(p.pos_b[0]-p.pos_a[0], p.pos_b[1]-p.pos_a[1]).norm();

            let alpha = movement::wall_orientation(&p) - f32::consts::PI / 2.0;
            
            let size = 2.0; // TODO

            let rot_mat = Mat2::new(alpha.cos(), -alpha.sin(),
                                    alpha.sin(), alpha.cos());
            let scale_mat = Mat2::new(w, 0.0,
                                      0.0, size);
            let m = rot_mat * scale_mat;
            let o = m * Vec2::new(0.5, 0.0);
            let model_mat = Mat4::new(m.m11, m.m12, 0.0, p.pos_b.x + o.x,
                                      m.m21, m.m22, 0.0, p.pos_b.y + o.y,
                                      0.0, 0.0, 1.0, 0.0,
                                      0.0, 0.0, 0.5, 1.0);
            draw_list.push((0, DrawElement::Square,
                            DrawAttributes::new(Vec4::new(0.2, 0.2, 0.2, 1.0), model_mat)));
        }
    }
}

impl_cached_system!(Components, Services, DrawWallSystem, aspect);

impl Process for DrawWallSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
