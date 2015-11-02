use na::Vec2;

pub const EPSILON: f32 = 10e-12; // TODO: Epsilon

/// Checks for an intersection between the line segments [a,b] and [p,q].
/// If there is an intersection, returns 0 <= s <= 1 with
///     a + s*(b-a) = p + t*(q-p)       for some 0 <= t <= 1.
pub fn line_segments_intersection(a: Vec2<f32>, b: Vec2<f32>, p: Vec2<f32>, q: Vec2<f32>)
                                  -> Option<f32> {
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

    let x = b.x - a.x;
    let y = p.x - q.x;
    let z = p.x - a.x;

    let u = b.y - a.y;
    let v = p.y - q.y;
    let w = p.y - a.y;

    // Solve |x y| * |s| = |z| for s and t
    //       |u v|   |t|   |w|
    //
    // |s| = 1/det * |v -y| * |z|
    // |t|           |-u x|   |w|
    //
    // where det = x*v - y*u

    let det = x * v - y * u;

    if det.abs() == 0.0 /*< EPSILON*/ { // Matrix not invertible => no intersection
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
