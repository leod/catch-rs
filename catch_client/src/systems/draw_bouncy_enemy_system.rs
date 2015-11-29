use na::{Vec2, Mat2, Vec4, Mat4, Norm};
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
            draw_list.push((3, DrawElement::Circle, DrawAttributes::new(color, model_mat)));

            let owner = data.net_entity[entity].owner;
            if owner != NEUTRAL_PLAYER_ID {
                let player_entity = data.services.net_entities.get_player_entity(owner);
                if let Some(player_entity) = player_entity {
                    data.with_entity_data(&player_entity, |player_e, c| {
                        let d = c.position[player_e].p - c.position[entity].p;

                        let alpha = d.y.atan2(d.x);
                        let size = 2.0;

                        let rot_mat = Mat2::new(alpha.cos(), -alpha.sin(),
                                                alpha.sin(), alpha.cos());
                        let scale_mat = Mat2::new(d.norm(), 0.0,
                                                  0.0, size);
                        let m = rot_mat * scale_mat;
                        let o = m * Vec2::new(0.5, 0.0);
                        let model_mat = Mat4::new(m.m11, m.m12, 0.0, c.position[entity].p.x + o.x,
                                                  m.m21, m.m22, 0.0, c.position[entity].p.y + o.y,
                                                  0.0, 0.0, 1.0, 0.0,
                                                  0.0, 0.0, 0.5, 1.0);
                        draw_list.push((4, DrawElement::Square,
                                        DrawAttributes::new(Vec4::new(0.0, 0.0, 1.0, 0.5), model_mat)));
                    });
                }
            }
        }
    }
}

impl_cached_system!(Components, Services, DrawBouncyEnemySystem, aspect);

impl Process for DrawBouncyEnemySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
