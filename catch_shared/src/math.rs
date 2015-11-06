use na::{Vec2, Mat2, Norm, Dot};

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

pub fn line_segment_sphere_intersection(a: Vec2<f32>, b: Vec2<f32>,
                                        c: Vec2<f32>, r: f32)
                                        -> Option<f32> {
    // We are looking for points on the line from a to b,
    // that also happen to be on the circle, i.e.:
    //     (x_1 - c_1)^2 + (x_2 - c_2)^2 = r^2
    // with x = a + s * b, where 0 <= s <= 1:
    //     (a_1 + sb_1 - c_1)^2 + (a_2 + sb_2 - c_2)^2 = r^2
    //                      <=> 
    //     (b_1^2+b_2^2)s^2 + 2((a_1-c_1)b_1 + (a_2-c_2)b_2)s + (a_1-c_1)^2 + (a_2-c_2)^2 - r^2 = 0

    let d = a - c;
    let u = b.sqnorm();
    let v = d.dot(&b);
    let w = d.sqnorm() - r*r;

    // Solve us^2 + vs + w = 0
    let det = v*v - 4.0*u*w;

    if det == 0.0 { // TODO: Epsilon?
        let s = -v / (2.0 * u);
        if s >= 0.0 && s <= 1.0 {
            Some(s)
        } else {
            None
        }
    } else if det > 0.0 {
        let det_sqrt = det.sqrt();
        let s1 = (-v + det_sqrt) / (2.0 * u);
        let s2 = (-v - det_sqrt) / (2.0 * u);
        let s1_in_range = s1 >= 0.0 && s1 <= 1.0;
        let s2_in_range = s2 >= 0.0 && s2 <= 1.0;

        if s1_in_range && s2_in_range {
            Some(s1.min(s2))
        } else if s1_in_range {
            Some(s1) 
        } else if s2_in_range {
            Some(s2)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn rect_sphere_overlap(p_rect: Vec2<f32>, w: f32, h: f32, angle: f32,
                           p_sphere: Vec2<f32>, r: f32)
                           -> bool {
    let rot_mat = Mat2::new(angle.cos(), -angle.sin(),
                            angle.sin(), angle.cos());
    let u = rot_mat * Vec2::new(0.5, 0.0) * w;
    let v = rot_mat * Vec2::new(0.0, 0.5) * h;

    // Corners of the rotated rectangle (clockwise)
    let p1 = p_rect - u - v; // upper left corner
    let p2 = p_rect + u - v;
    let p3 = p_rect + u + v;
    let p4 = p_rect - u + v;

    let s1 = line_segment_sphere_intersection(p1, p2, p_sphere, r);
    let s2 = line_segment_sphere_intersection(p2, p3, p_sphere, r);
    let s3 = line_segment_sphere_intersection(p3, p4, p_sphere, r);
    let s4 = line_segment_sphere_intersection(p4, p1, p_sphere, r);

    s1.is_some() || s2.is_some() || s3.is_some() || s4.is_some()
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
