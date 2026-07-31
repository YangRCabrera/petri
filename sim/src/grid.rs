//! Coordinate helpers for the simulation grid.

/// Wraps `coord` into `[0, max_size)`, giving the grid toroidal (wrap-around)
/// boundaries instead of hard edges.
pub(crate) fn wrap_coordinate(coord: isize, max_size: usize) -> usize {
    coord.rem_euclid(max_size as isize) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_within_bounds_is_unchanged() {
        assert_eq!(wrap_coordinate(4, 10), 4);
        assert_eq!(wrap_coordinate(0, 10), 0);
    }

    #[test]
    fn negative_coordinate_wraps_from_the_end() {
        assert_eq!(wrap_coordinate(-1, 10), 9);
        assert_eq!(wrap_coordinate(-11, 10), 9);
    }

    #[test]
    fn coordinate_past_the_end_wraps_to_the_start() {
        assert_eq!(wrap_coordinate(10, 10), 0);
        assert_eq!(wrap_coordinate(13, 10), 3);
    }
}
