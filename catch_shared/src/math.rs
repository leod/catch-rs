use vecmath_lib;

pub use vecmath_lib::vec2_dot as dot;
pub use vecmath_lib::vec2_neg as neg;
pub use vecmath_lib::vec2_cross as cross;
pub use vecmath_lib::vec2_add as add;
pub use vecmath_lib::vec2_sub as sub;
pub use vecmath_lib::vec2_mul as mul;
pub use vecmath_lib::vec2_scale as scale;
pub use vecmath_lib::vec2_square_len as square_len;
pub use vecmath_lib::vec2_normalized as normalized;

pub type Scalar = f32;
pub type Matrix2 = vecmath_lib::Matrix2x3<Scalar>;
pub type Vec2 = vecmath_lib::Vector2<Scalar>;

pub const EPSILON: Scalar = 10e-5; // TODO: Epsilon

/// Checks for an intersection between the line segments [a,b] and [p,q].
/// If there is an intersection, returns 0 <= s <= 1 with
///     a + s*(b-a) = p + t*(q-p)       for some 0 <= t <= 1.
pub fn line_segments_intersection(a: Vec2, b: Vec2, p: Vec2, q: Vec2) -> Option<f32> {
    // a + s*(b-a) = p + t*(q-p)
    //          <=>
    // s*(b-a) - t*(q-p) = p-a 
    //          <=>
    // s*(b[0]-a[0]) - t*(q[0]-p[0]) = p[0]-a[0] 
    //          and
    // s*(b[1]-a[1]) - t*(q[1]-p[1]) = p[1]-a[1] 
    //          <=>
    // |b[0]-a[0]    p[0]-q[0]| |s| = |p[0]-a[0]|
    // |b[1]-a[1]    p[1]-q[1]| |t| = |p[1]-a[1]|

    let x = b[0] - a[0];
    let y = p[0] - q[0];
    let z = p[0] - a[0];

    let u = b[1] - a[1];
    let v = p[1] - q[1];
    let w = p[1] - a[1];

    // Solve |x y| * |s| = |z| for s and t
    //       |u v|   |t|   |w|
    //
    // |s| = 1/det * |v -y| * |z|
    // |t|           |-u x|   |w|
    //
    // where det = x*v - y*u

    let det = x * v - y * u;

    if det.abs() < EPSILON { // Matrix not invertible => no intersection
        return None;
    }

    let inv_det = 1.0 / det;

    let s = inv_det * (v * z - y * w);
    let t = inv_det * (x * w - u * z);

    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
        Some(s)
    } else {
        None
    }
}

pub fn min_intersection<T>(a: Option<(T, f32)>, b: Option<(T, f32)>) -> Option<(T, f32)> {
    match (a, b) {
        (Some((x, s)), Some((y, t))) => 
            if s < t { Some((x, s)) } else { Some((y, t)) },
        (Some((x, s)), None) => Some((x, s)),
        (None, Some((y, t))) => Some((y, t)),
        (None, None) => None
    }
}
