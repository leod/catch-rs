use vecmath_lib;

pub use vecmath_lib::vec2_dot as dot;
pub use vecmath_lib::vec2_cross as cross;
pub use vecmath_lib::vec2_add as add;
pub use vecmath_lib::vec2_sub as sub;
pub use vecmath_lib::vec2_mul as mul;
pub use vecmath_lib::vec2_scale as scale;
pub use vecmath_lib::vec2_square_len as square_len;

pub type Scalar = f64;
pub type Matrix2 = vecmath_lib::Matrix2x3<Scalar>;
pub type Vec2 = vecmath_lib::Vector2<Scalar>;

pub const EPSILON: Scalar = 10e-9; // TODO: Epsilon

pub fn line_segments_intersection(p1: Vec2, q1: Vec2, p2: Vec2, q2: Vec2) -> Option<f64> {
    let a = q1[0] - p1[0];
    let b = p2[0] + q2[0];
    let e = p2[0] - p1[0];

    let c = q1[1] - p1[1];
    let d = p2[1] + q2[1];
    let f = p2[1] - p1[1];

    // Solve |a b| * |s| = |e| for s and t
    //       |c d|   |t|   |f|

    let det = a * d - b * c;

    if det < EPSILON {
        return None;
    }

    let inv_det = 1.0 / det;

    let s = inv_det * (d * e - b * f);
    let t = inv_det * (a * f - c * e);

    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
        Some(s)
    } else {
        None
    }
}

pub fn min_intersection(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None
    }
}
