//! Implementation of the midpoint circle drawing algorithm
//! https://en.wikipedia.org/wiki/Midpoint_circle_algorithm

use itertools::Itertools;

/// Computes all the points of a circle at the origin with a given radius
pub fn circle_points(radius: u32) -> Vec<(i32, i32)> {
    let segment = circle_segment(radius);

    segment
        .iter()
        .flat_map(|point| project_point_eight_ways(*point))
        .unique()
        .collect()
}

/// Gives the circle segment that lies in the 90° to 45° 8-slice.
fn circle_segment(radius: u32) -> Vec<(i32, i32)> {
    // add initial point at 90° degrees
    let mut segment = vec![(0, radius as i32)];

    // add remaining points in 90° to 45° slice
    let (mut point_x, mut point_y) = segment[0];
    loop {
        // calculate next point
        let (mid_x, mid_y) = (point_x as f32 + 1.0, point_y as f32 - 0.5);

        if mid_x.powf(2.0) + mid_y.powf(2.0) > radius.pow(2) as f32 {
            point_y -= 1;
        }
        point_x += 1;

        // check if point is still in 90° to 45° range
        if point_x > point_y {
            break;
        }

        // add point to segment
        segment.push((point_x, point_y));
    }
    segment
}

/// Takes a point in the 90° to 45° slice and projects it to the other 8-slices
fn project_point_eight_ways((x, y): (i32, i32)) -> [(i32, i32); 8] {
    assert!(
        x >= 0 && y >= 0 && x <= y,
        "point must lie in the 90° to 45° slice of the xy-plane!"
    );

    [
        (x, y),
        (y, x),
        (y, -x),
        (x, -y),
        (-x, -y),
        (-y, -x),
        (-y, x),
        (-x, y),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_segment_radius_2() {
        //    --
        //    21012
        // -2 _███_
        // -1 █___█
        //  0 █___█
        //  1 █___█
        //  2 _███_
        let radius = 2;

        let actual_segment = circle_segment(radius);

        //    --
        //    21012
        //  1 _____
        //  2 __██_
        let expected_segment = [(0, 2), (1, 2)];
        assert_eq!(&actual_segment, &expected_segment);
    }

    #[test]
    fn circle_segment_radius_3() {
        //    ---
        //    3210123
        // -3 __███__
        // -2 _█___█_
        // -1 █_____█
        //  0 █_____█
        //  1 █_____█
        //  2 _█___█_
        //  3 __███__
        let radius = 3;

        let actual_segment = circle_segment(radius);

        //    ---
        //    3210123
        //  2 _____█_
        //  3 ___██__
        let expected_segment = [(0, 3), (1, 3), (2, 2)];
        assert_eq!(&actual_segment, &expected_segment);
    }

    #[test]
    fn circle_segment_radius_4() {
        //    ----
        //    432101234
        // -4 ___███___
        // -3 _██___██_
        // -2 _█_____█_
        // -1 █_______█
        //  0 █_______█
        //  1 █_______█
        //  2 _█_____█_
        //  3 _██___██
        //  4 ___███__
        let radius = 4;

        let actual_segment = circle_segment(radius);

        //    ----
        //    432101234
        //  3 ______██
        //  4 ____██__
        let expected_segment = [(0, 4), (1, 4), (2, 3), (3, 3)];
        assert_eq!(&actual_segment, &expected_segment);
    }

    #[test]
    fn circle_segment_radius_5() {
        //    -----
        //    54321012345
        // -5 ___█████___
        // -4 __█_____█__
        // -3 _█_______█_
        // -2 █_________█
        // -1 █_________█
        //  0 █_________█
        //  1 █_________█
        //  2 █_________█
        //  3 _█_______█_
        //  4 __█_____█__
        //  5 ___█████___
        let radius = 5;

        let actual_segment = circle_segment(radius);

        //    -----
        //    54321012345
        //  4 ________█__
        //  5 _____███___
        let expected_segment = [(0, 5), (1, 5), (2, 5), (3, 4)];
        assert_eq!(&actual_segment, &expected_segment);
    }

    #[test]
    fn circle_radius_2() {
        //    --
        //    21012
        // -2 _███_
        // -1 █___█
        //  0 █___█
        //  1 █___█
        //  2 _███_
        let radius = 2;

        let actual_points = circle_points(radius)
            .into_iter()
            .sorted()
            .collect::<Vec<(i32, i32)>>();

        let expected_points = [
            (-2, -1),
            (-2, 0),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (0, -2),
            (0, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 0),
            (2, 1),
        ]
        .into_iter()
        .sorted()
        .collect::<Vec<(i32, i32)>>();

        assert_eq!(&actual_points, &expected_points);
    }
}
