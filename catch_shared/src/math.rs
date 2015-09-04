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
