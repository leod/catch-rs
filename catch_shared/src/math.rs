use na::{Vec2, Mat2, Norm, Dot};

pub const EPSILON: f32 = 10e-12; // TODO: Epsilon

/// If existing, returns s >= 0 and t >= 0 with: a + s*d = p + t*v
pub fn ray_ray_intersection(a: Vec2<f32>, d: Vec2<f32>, p: Vec2<f32>, v: Vec2<f32>)
                            -> Option<(f32, f32)> {
    // a + s*d = p + t*v
    //          <=>
    // s*d - t*v = p-a 
    //          <=>
    // |d.x -v.x| |s| = |p.x-a.x|
    // |d.y -v.y| |t| = |p.y-a.y|

    let z = p.x - a.x;
    let w = p.y - a.y;

    // Solve |d.x -v.x| * |s| = |z| for s and t
    //       |d.y -v.y|   |t|   |w|
    //
    // |s| = 1/det * |-v.y v.x| * |z|
    // |t|           |-d.y d.x|   |w|

    let det = d.x * -v.y + v.x * d.y;

    if det.abs() == 0.0 /*< EPSILON*/ { // Matrix not invertible => no intersection
        return None;
    }

    let inv_det = 1.0 / det;

    let s = inv_det * (-v.y * z + v.x * w);
    let t = inv_det * (d.x * w - d.y * z);

    if s >= 0.0 && t >= 0.0 {
        Some((s, t))
    } else {
        None
    }
}

pub fn ray_line_segment_intersection(a: Vec2<f32>, d: Vec2<f32>, p: Vec2<f32>, q: Vec2<f32>)
                                     -> Option<(f32, f32)> {
    match ray_ray_intersection(a, d, p, q - p) {
        Some((s, t)) if t <= 1.0 => Some((s, t)),
        _ => None
    }
}

pub enum SecondDegZero {
    One(f32),
    Two(f32, f32),
    None,
}

impl SecondDegZero {
    /// Returns the smallest zero in range [min,max], if there is one
    pub fn min_in_range(&self, min: f32, max: f32) -> Option<f32> {
        match *self {
            SecondDegZero::One(s) => {
                if s >= min && s <= max {
                    Some(s)
                } else {
                    None
                }
            }
            SecondDegZero::Two(s1, s2) => { 
                let s1_in_range = s1 >= min && s1 <= max;
                let s2_in_range = s2 >= min && s2 <= max;

                if s1_in_range && s2_in_range {
                    Some(s1.min(s2))
                } else if s1_in_range {
                    Some(s1) 
                } else if s2_in_range {
                    Some(s2)
                } else {
                    None
                }
            }
            SecondDegZero::None => None
        }
    }
}

/// Solve ux^2 + vx + w = 0 for `x`
pub fn solve_second_deg(u: f32, v: f32, w: f32) -> SecondDegZero {
    let det = v*v - 4.0*u*w;

    if det == 0.0 {
        SecondDegZero::One(-v / (2.0 * u))
    } else if det > 0.0 {
        let det_sqrt = det.sqrt();

        if v >= 0.0 {
            SecondDegZero::Two((-v - det_sqrt) / (2.0 * u),
                               (2.0 * w) / (-v - det_sqrt))
        } else {
            SecondDegZero::Two((2.0 * w) / (-v + det_sqrt),
                               (-v + det_sqrt) / (2.0 * u))
        }
    } else {
        SecondDegZero::None
    }

    /*let det = v*v - 4.0*u*w;

    if det == 0.0 { // TODO: Epsilon?
        SecondDegZero::One(-v / (2.0 * u))
    } else if det > 0.0 {
        let det_sqrt = det.sqrt();
        SecondDegZero::Two((-v + det_sqrt) / (2.0 * u),
                           (-v - det_sqrt) / (2.0 * u))
    } else {
        SecondDegZero::None
    }*/
}

/// If the line segment a+b*s (0 <= `s` <= 1) and the circle at position `c` with radius `r`
/// intersect, returns the smallest `s` such that a+b*s is an intersection point
pub fn line_segment_circle_intersection(a: Vec2<f32>, b: Vec2<f32>,
                                        c: Vec2<f32>, r: f32)
                                        -> Option<f32> {
    // TODO: Ignores the case that the line is completely inside the circle

    // |x - c| = r
    // |a + s*(b-a) - c| = r
    // (b-a)^2*s^2 + 2*(a-c)*(b-a)*s + (a-c)^2 = r^2

    let u = (b - a).sqnorm();
    let v = 2.0 * (a - c).dot(&(b - a));
    let w = (a - c).sqnorm() - r*r;

    // Solve us^2 + vs + w = 0 for the smallest s with 0 <= s <= 1
    solve_second_deg(u, v, w).min_in_range(0.0, 1.0)
}

fn project_onto(v: Vec2<f32>, p: Vec2<f32>) -> Vec2<f32> {
    p * (v.dot(&p) / p.sqnorm())
}

fn perp_onto(v: Vec2<f32>, p: Vec2<f32>) -> Vec2<f32> {
    v - project_onto(v, p)
}

pub fn lerp_project_point_onto_line(p: Vec2<f32>, a: Vec2<f32>, b: Vec2<f32>) -> f32 {
    let l = p - a;
    let d = b - a;
    l.dot(&d) / d.dot(&d)
}

pub fn point_line_segment_distance(p: Vec2<f32>, a: Vec2<f32>, b: Vec2<f32>) -> f32 {
    let s = lerp_project_point_onto_line(p, a, b);
    //println!("point on line: {:?}, s: {}", a+(b-a)*s, s);
    if s < 0.0 {
        (p - a).norm()
    } else if s > 1.0 {
        (p - b).norm()
    } else {
        (p - (a + (b - a)*s)).norm()
    }
}

pub fn line_moving_circle_intersection_time(a: Vec2<f32>, b: Vec2<f32>,
                                            c: Vec2<f32>, d: Vec2<f32>, r: f32)
                                            -> Option<f32> {
    let x = (a.y - b.y)*d.x - (a.x - b.x)*d.y;
    let y = (a.x - c.x)*(a.y - b.y) - (a.x - b.x)*(a.y - c.y);
    let u = x*x;
    let v = -2.0 * x * y;
    let w = y*y - r*r*(b - a).sqnorm();

    if w <= 0.0 {
        Some(0.0)
    } else {
        solve_second_deg(u, v, w).min_in_range(0.0, 1.0)
    }
}

pub fn line_segment_moving_circle_intersection_time(a: Vec2<f32>, b: Vec2<f32>,
                                                    c: Vec2<f32>, d: Vec2<f32>, r: f32)
                                                    -> Option<f32> {
    let check = |t| {
        match t {
            Some(t) => {
                //println!("point on: {:?}", c + d*t);
                let distance = point_line_segment_distance(c + d*t, a, b);
                //println!("distance to point: {}, t: {}", distance, t);
                if distance <= r + 0.001 {
                    Some(t)
                } else {
                    None
                }
            }
            None => None
        }
    };

    //println!("LINE");
    let t_line = check(line_moving_circle_intersection_time(a, b, c, d, r));
    //println!("AA");
    let t_a = check(point_moving_circle_intersection_time(a, c, d, r));
    let t_a = point_moving_circle_intersection_time(a, c, d, r);
    //println!("BB");
    let t_b = check(point_moving_circle_intersection_time(b, c, d, r));
    let t_b = point_moving_circle_intersection_time(b, c, d, r);
    //println!("{:?} {:?} {:?}", t_line, t_a, t_b);

    min_option(t_line, min_option(t_a, t_b))
}

pub fn point_moving_circle_intersection_time(p: Vec2<f32>, c: Vec2<f32>, d: Vec2<f32>, r: f32)
                                             -> Option<f32> {
    // |(c + td) - p| = r
    // d^2*t^2 + 2d(c-p)*t + (c-p)^2 = r^2

    let u = d.sqnorm();
    let v = 2.0 * d.dot(&(c - p));
    let w = (c - p).sqnorm() - r*r;

    if w <= 0.0 {
        Some(0.0)
    } else {
        solve_second_deg(u, v, w).min_in_range(0.0, 1.0)
    }
}

pub fn moving_circles_intersection_time(a1: Vec2<f32>, v1: Vec2<f32>, r1: f32,
                                        a2: Vec2<f32>, v2: Vec2<f32>, r2: f32) -> Option<f32> {
    // untested 

    let b1 = a1 + v1;
    let b2 = a2 + v2;

    let u = b1.dot(&b1) + b2.dot(&b2);
    let v = 2.0 * (b1.dot(&a1) + b2.dot(&a2) - b2.dot(&a1) - b1.dot(&a2));
    let w = a1.dot(&a1) + a2.dot(&a2) - (r1 + r2) * (r1 + r2);

    solve_second_deg(u, v, w).min_in_range(0.0, 1.0)
}

/// Checks if the rectangle with center position `p_rect` and dimensions [`w`,`h`], when rotated by
/// `angle` radians, overlaps with the circle at position `p_circle` with radius `r`
pub fn rect_circle_overlap(p_rect: Vec2<f32>, w: f32, h: f32, angle: f32,
                           p_circle: Vec2<f32>, r: f32)
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

    line_segment_circle_intersection(p1, p2, p_circle, r).is_some() ||
    line_segment_circle_intersection(p2, p3, p_circle, r).is_some() ||
    line_segment_circle_intersection(p3, p4, p_circle, r).is_some() ||
    line_segment_circle_intersection(p4, p1, p_circle, r).is_some() 
}

pub fn min_option<T: PartialOrd>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(s), Some(t)) => 
            if s < t { Some(s) } else { Some(t) },
        (Some(s), None) => Some(s),
        (None, Some(t)) => Some(t),
        (None, None) => None
    }
}
