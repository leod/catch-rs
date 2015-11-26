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

/// Solve ux^2 + vx + w = 0 for x
pub fn solve_second_deg(u: f32, v: f32, w: f32) -> SecondDegZero {
    let det = v*v - 4.0*u*w;

    if det < 0.0 { return SecondDegZero::None; }

    let s0 = -v / (2.0 * u);
    let sd = det.sqrt() / (2.0 * u);

    if sd == 0.0 { SecondDegZero::One(s0 - sd) }
    else {
        SecondDegZero::Two(s0 - sd, s0 + sd)
    }

    /*if det == 0.0 { // TODO: Epsilon?
        SecondDegZero::One(-v / (2.0 * u))
    } else if det > 0.0 {
        let det_sqrt = det.sqrt();
        SecondDegZero::Two((-v + det_sqrt) / (2.0 * u),
                           (-v - det_sqrt) / (2.0 * u))
    } else {
        SecondDegZero::None
    }*/
}

/// Solve ux^2 + vx + w = 0 for the smallest x in [min,max]
pub fn solve_second_deg_min_in_range(u: f32, v: f32, w: f32, min: f32, max: f32) -> Option<f32> { 
    match solve_second_deg(u, v, w) {
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

pub fn line_segment_circle_intersection(a: Vec2<f32>, b: Vec2<f32>,
                                        c: Vec2<f32>, r: f32)
                                        -> Option<f32> {
    // |x - c| = r
    // |a + s*(b-a) - c| = r
    // (b-a)^2*s^2 + 2*(a-c)*(b-a)*s + (a-c)^2 = r^2

    let u = (b - a).sqnorm();
    let v = 2.0 * (a - c).dot(&(b - a));
    let w = (a - c).sqnorm() - r*r;

    // Solve us^2 + vs + w = 0 for the smallest s with 0 <= s <= 1
    solve_second_deg_min_in_range(u, v, w, 0.0, 1.0)
}

fn project_onto(v: Vec2<f32>, p: Vec2<f32>) -> Vec2<f32> {
    p * (v.dot(&p) / p.sqnorm())
}

fn perp_onto(v: Vec2<f32>, p: Vec2<f32>) -> Vec2<f32> {
    v - project_onto(v, p)
}

//pub fn line_segment_point_distance

pub fn line_segment_moving_circle_intersection(a: Vec2<f32>, b: Vec2<f32>,
                                               c: Vec2<f32>, d: Vec2<f32>, r: f32)
                                               -> Option<f32> {
    let e = (b - a).normalize();
    let u = e.x*d.x - e.y*d.y;
    let w = e.x*c.x - e.y*c.y + b.x*a.y - a.x*b.y;
    let v = 2.0*u*w;

    let i = line_segment_circle_intersection(a,b,c,r);
    if let Some(i) = i { info!("xx: {:?}", i); }
    //info!("{} x^2 + {} x + {} = 0", u*u, v, w*w - r*r);

    /*let u = perp_onto(d, e);
    let w = perp_onto(c - a, e);
    let v = 2.0*u.dot(&w);

    let u = (d - e * e.dot(&d)).dot(&d);
    let v = 2.0 * (d - e * e.dot(&d)).dot(&(c - a));
    let w = (c - a - e * e.dot(&(c - a))).dot(&(c - a)) - r*r;*/

    /*
       det = ax^2 + ay^2 - 2*ax*bx + bx^2 - 2*ay*by + by^2
     
       t == (ay*bx - ax*by - (ay - by)*cx + (ax - bx)*cy - sqrt(det)*r)/((ay - by)*dx - (ax - bx)*dy),
       t == (ay*bx - ax*by - (ay - by)*cx + (ax - bx)*cy + sqrt(det)*r)/((ay - by)*dx - (ax - bx)*dy)

    */

    /*let det = a.x * a.x + a.y * a.y - 2.0 * a.x * b.x + b.x * b.x - 2.0 * a.y * b.y + b.y * b.y;
    let c1 = a.y * b.x - a.x * b.y - (a.y - b.y) * c.x + (a.x - b.x) * c.y;
    let c2 = det.sqrt() * r;
    let c3 = */
    
    let u = ((a.y - b.y)*d.x - (a.x - b.x)*d.y);
    //let v = -2.0 * ((a.x - c.x)*(a.y - b.y) - (a.x - b.x)*(a.y - c.y))*((a.y - b.y)*d.x - (a.x - b.x) * d.y);
    let w = ((a.x - c.x)*(a.y - b.y) - (a.x - b.x)*(a.y - c.y));
    let v = -2.0 * u * w;

    match solve_second_deg_min_in_range(u*u, v, w*w  - r*r, 0.0, 10000000.0) {
        Some(t) => {
            //info!("{} x^2 + {} x + {} = 0", u.sqnorm(), v, w.sqnorm() - r*r);
            info!("t: {}", t);
            Some(t)

            //if c + d * t

            //return None;
            /*match line_segment_circle_intersection(a, b,
                                                   c + d*t, r) {
                Some(_) => Some(t),
                None => None,
            }*/
        }
        None => None
    }
}

pub fn moving_circles_intersection(a1: Vec2<f32>, v1: Vec2<f32>, r1: f32,
                                   a2: Vec2<f32>, v2: Vec2<f32>, r2: f32) -> Option<f32> {
    // untested 

    let b1 = a1 + v1;
    let b2 = a2 + v2;

    let u = b1.dot(&b1) + b2.dot(&b2);
    let v = 2.0 * (b1.dot(&a1) + b2.dot(&a2) - b2.dot(&a1) - b1.dot(&a2));
    let w = a1.dot(&a1) + a2.dot(&a2) - (r1 + r2) * (r1 + r2);

    solve_second_deg_min_in_range(u, v, w, 0.0, 1.0)
}

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

pub fn min_intersection<T>(a: Option<(T, f32)>, b: Option<(T, f32)>) -> Option<(T, f32)> {
    match (a, b) {
        (Some((x, s)), Some((y, t))) => 
            if s < t { Some((x, s)) } else { Some((y, t)) },
        (Some((x, s)), None) => Some((x, s)),
        (None, Some((y, t))) => Some((y, t)),
        (None, None) => None
    }
}
