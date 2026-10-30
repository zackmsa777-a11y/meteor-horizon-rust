//! Exact ray parameter intervals through loaded columns; no reach cap.
pub(super) fn columns(
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    columns: impl IntoIterator<Item = (i32, i32)>,
    min_y: i32,
    height: i32,
) -> Vec<(f64, f64)> {
    let mut intervals = Vec::new();
    for (x, z) in columns {
        let lo = [f64::from(x) * 16.0, f64::from(min_y), f64::from(z) * 16.0];
        let hi = [lo[0] + 16.0, lo[1] + f64::from(height), lo[2] + 16.0];
        let (mut start, mut end) = (0.0f64, reach);
        let mut miss = false;
        for axis in 0..3 {
            if dir[axis] == 0.0 {
                if origin[axis] < lo[axis] || origin[axis] >= hi[axis] {
                    miss = true;
                    break;
                }
            } else {
                let a = (lo[axis] - origin[axis]) / dir[axis];
                let b = (hi[axis] - origin[axis]) / dir[axis];
                start = start.max(a.min(b));
                end = end.min(a.max(b));
            }
        }
        if !miss && start <= end && start.is_finite() && end.is_finite() {
            intervals.push((start, end));
        }
    }
    intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for (start, end) in intervals {
        if let Some(last) = merged.last_mut() {
            if start <= last.1 {
                last.1 = last.1.max(end);
                continue;
            }
        }
        merged.push((start, end));
    }
    merged
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn axes_inside_and_entering() {
        for axis in 0..3 {
            for sign in [-1.0, 1.0] {
                let mut dir = [0.0; 3];
                dir[axis] = sign;
                assert_eq!(
                    columns([8.0; 3], dir, f64::INFINITY, [(0, 0)], 0, 16),
                    vec![(0.0, 8.0)]
                );
                let mut origin = [8.0; 3];
                origin[axis] = if sign > 0.0 { -8.0 } else { 24.0 };
                assert_eq!(
                    columns(origin, dir, f64::MAX, [(0, 0)], 0, 16),
                    vec![(8.0, 24.0)]
                );
                dir[axis] = -sign;
                assert!(columns(origin, dir, f64::MAX, [(0, 0)], 0, 16).is_empty());
            }
        }
    }
    #[test]
    fn unloaded_gaps_adjacent_and_negative_columns() {
        assert_eq!(
            columns(
                [-24.0, 8.0, 8.0],
                [1.0, 0.0, 0.0],
                f64::MAX,
                [(-1, 0), (0, 0), (1000000, 0)],
                0,
                16
            ),
            vec![(8.0, 40.0), (16000024.0, 16000040.0)]
        );
        assert!(columns([0.0; 3], [1.0, 0.0, 0.0], f64::INFINITY, [], 0, 16).is_empty());
        assert!(
            columns(
                [8.0, 20.0, 8.0],
                [1.0, 0.0, 0.0],
                f64::INFINITY,
                [(0, 0)],
                0,
                16
            )
            .is_empty()
        );
    }
}
