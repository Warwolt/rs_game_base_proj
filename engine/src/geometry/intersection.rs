use super::{Point, Rect};

pub fn point_is_inside_rect(point: Point, rect: Rect) -> bool {
    let rect_x0 = rect.x;
    let rect_y0 = rect.y;
    let rect_x1 = rect.x + rect.w as i32;
    let rect_y1 = rect.y + rect.h as i32;
    let (point_x, point_y) = (point.x as i32, point.y as i32);

    let horizontal_overlap = rect_x0 <= point_x && point_x <= rect_x1;
    let vertical_overlap = rect_y0 <= point_y && point_y <= rect_y1;

    horizontal_overlap && vertical_overlap
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{point, rect};

    #[test]
    fn point_outside_rect() {
        //   0 1 2 3
        // 0 ┌───┐
        // 1 │   │ o
        // 2 └───┘
        let rect = rect(0, 0, 2, 2);
        let point = point(1, 3);
        assert!(!point_is_inside_rect(point, rect))
    }

    #[test]
    fn point_on_rect_side() {
        //   0 1 2 3
        // 0 ┌───┐
        // 1 │   o
        // 2 └───┘
        let rect = rect(0, 0, 2, 2);
        let point = point(1, 2);
        assert!(point_is_inside_rect(point, rect))
    }

    #[test]
    fn point_inside_rect() {
        //   0 1 2 3
        // 0 ┌───┐
        // 1 │ o │
        // 2 └───┘
        let rect = rect(0, 0, 2, 2);
        let point = point(1, 1);
        assert!(point_is_inside_rect(point, rect))
    }
}
