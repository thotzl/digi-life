//! Platform-deterministic f32 math for the per-tick simulation path.
//! The std transcendental functions call the platform libm, whose results may differ in the last
//! ulp between operating systems; the pure-Rust `libm` crate gives identical results everywhere.
//! `sqrt` is IEEE-exact and stays on std.

#[inline]
pub fn sin(x: f32) -> f32 {
    libm::sinf(x)
}

#[inline]
pub fn cos(x: f32) -> f32 {
    libm::cosf(x)
}

#[inline]
pub fn atan2(y: f32, x: f32) -> f32 {
    libm::atan2f(y, x)
}

#[inline]
pub fn exp(x: f32) -> f32 {
    libm::expf(x)
}

#[inline]
pub fn tanh(x: f32) -> f32 {
    libm::tanhf(x)
}

#[inline]
pub fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}
