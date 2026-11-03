//! Plane geometry helpers shared by Ocean, trainer, sensing and probes. Each helper is the exact expression it
//! replaced (`dx * dx + dy * dy`, the `while` wrap loop), so results are bit-identical.

use std::f32::consts::{PI, TAU};

/// Squared distance between `(ax, ay)` and `(bx, by)`.
#[inline]
pub fn dist_sq(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (bx - ax, by - ay);
    dx * dx + dy * dy
}

/// Distance between `(ax, ay)` and `(bx, by)`.
#[inline]
pub fn dist(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    dist_sq(ax, ay, bx, by).sqrt()
}

/// Angle wrapped into `[-π, π]` by whole turns.
#[inline]
pub fn wrap_angle(mut a: f32) -> f32 {
    while a > PI {
        a -= TAU;
    }
    while a < -PI {
        a += TAU;
    }
    a
}

/// Offset `(r · cos θ, r · sin θ)` of a point at distance `radius` and angle `angle` (`fmath`, platform-independent).
#[inline]
pub fn polar_offset(radius: f32, angle: f32) -> (f32, f32) {
    (radius * super::fmath::cos(angle), radius * super::fmath::sin(angle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_and_wrap() {
        assert_eq!(dist(0.0, 0.0, 3.0, 4.0), 5.0);
        assert_eq!(dist_sq(1.0, 1.0, -2.0, 5.0), 25.0);
        assert!((wrap_angle(3.0 * PI) - PI).abs() < 1e-5);
        assert!((wrap_angle(-2.5 * PI) + 0.5 * PI).abs() < 1e-5);
        assert_eq!(wrap_angle(0.5), 0.5);
        let (x, y) = polar_offset(2.0, PI / 2.0);
        assert!(x.abs() < 1e-6 && (y - 2.0).abs() < 1e-6);
    }
}
