use ecs::{Aspect, System, DataHelper, Process};
use na::Vec4;

use shared::util::CachedAspect;

use components::Components;
use services::Services;
use draw::DrawList;

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
            draw_list.push_line(Vec4::new(0.2, 0.2, 0.2, 1.0), 2.0, p.pos_a, p.pos_b, 0.0);
        }
    }
}

impl_cached_system!(Components, Services, DrawWallSystem, aspect);

impl Process for DrawWallSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}
