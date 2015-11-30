use std::slice;

use na::{Norm, Vec2, Vec4, Mat2, Mat4};

#[derive(Clone, Debug)]
pub enum DrawElement {
    Circle,
    Square,
    TexturedSquare { texture: String },
}

#[derive(Copy, Clone, Debug)]
pub struct DrawAttributes {
    pub z: f32,
    pub color: Vec4<f32>,
    pub model_mat: Mat4<f32>,
    pub texture_id: u32,
}
implement_vertex!(DrawAttributes, z, color, model_mat, texture_id);

impl DrawAttributes {
    pub fn new(z: f32, color: Vec4<f32>, model_mat: Mat4<f32>) -> DrawAttributes {
        DrawAttributes {
            z: z,
            color: color,
            model_mat: model_mat,
            texture_id: 0,
        }
    }
}

pub struct DrawList {
    list: Vec<(DrawElement, DrawAttributes)>,
}

impl DrawList {
    pub fn new() -> DrawList {
        DrawList {
            list: Vec::new()
        }
    }

    pub fn iter(&self) -> slice::Iter<(DrawElement, DrawAttributes)> {
        self.list.iter()
    }

    pub fn sort_by_z(&mut self) {
        self.list.sort_by(|a, b| a.1.z.partial_cmp(&b.1.z).unwrap());
    }

    pub fn push(&mut self, element: DrawElement, attributes: DrawAttributes) {
        self.list.push((element, attributes));
    }

    pub fn push_line(&mut self,
                     color: Vec4<f32>,
                     size: f32,
                     a: Vec2<f32>,
                     b: Vec2<f32>,
                     z: f32) {
        let d = b - a;
        let alpha = d.y.atan2(d.x);

        let rot_mat = Mat2::new(alpha.cos(), -alpha.sin(),
                                alpha.sin(), alpha.cos());
        let scale_mat = Mat2::new(d.norm(), 0.0,
                                  0.0, size);
        let m = rot_mat * scale_mat;
        let o = m * Vec2::new(0.5, 0.0);
        let model_mat = Mat4::new(m.m11, m.m12, 0.0, a.x + o.x,
                                  m.m21, m.m22, 0.0, a.y + o.y,
                                  0.0, 0.0, 1.0, 0.0,
                                  0.0, 0.0, 0.5, 1.0);
        self.push(DrawElement::Square, DrawAttributes::new(z, color, model_mat));
    }

    pub fn push_rect(&mut self,
                     color: Vec4<f32>,
                     width: f32,
                     height: f32,
                     p: Vec2<f32>,
                     z: f32,
                     angle: f32) {
        let rot_mat = Mat2::new(angle.cos(), -angle.sin(),
                                angle.sin(), angle.cos());
        let scale_mat = Mat2::new(width, 0.0,
                                  0.0, height);
        let m = rot_mat * scale_mat;
        let model_mat = Mat4::new(m.m11, m.m12, 0.0, p.x,
                                  m.m21, m.m22, 0.0, p.y,
                                  0.0, 0.0, 1.0, 0.0,
                                  0.0, 0.0, 0.0, 1.0);
        self.push(DrawElement::Square, DrawAttributes::new(z, color, model_mat));
    }

    pub fn push_ellipse(&mut self,
                        color: Vec4<f32>,
                        width: f32,
                        height: f32,
                        p: Vec2<f32>,
                        z: f32,
                        angle: f32) {
        let rot_mat = Mat2::new(angle.cos(), -angle.sin(),
                                angle.sin(), angle.cos());
        let scale_mat = Mat2::new(width, 0.0,
                                  0.0, height);
        let m = rot_mat * scale_mat;
        let model_mat = Mat4::new(m.m11, m.m12, 0.0, p.x,
                                  m.m21, m.m22, 0.0, p.y,
                                  0.0, 0.0, 1.0, 0.0,
                                  0.0, 0.0, 0.0, 1.0);
        self.push(DrawElement::Circle, DrawAttributes::new(z, color, model_mat));
    }
}

