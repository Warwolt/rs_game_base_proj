pub mod intersection;

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Dimension {
    pub width: u32,
    pub height: u32,
}

pub type Point = glam::IVec2;

#[inline(always)]
pub const fn rect(x: i32, y: i32, w: u32, h: u32) -> Rect {
    Rect { x, y, w, h }
}

#[inline(always)]
pub const fn point(x: i32, y: i32) -> Point {
    glam::IVec2::new(x, y)
}
