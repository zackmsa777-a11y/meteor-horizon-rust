// SPDX-License-Identifier: GPL-3.0-only
// Source-derived single-selection Meteor Snapper. Copyright Meteor Development.
use crate::hud_box::HudBox;
#[derive(Clone, Copy)]
enum Direction {
    Right,
    Left,
    Top,
    Bottom,
}
#[derive(Default)]
pub struct Snapper {
    to: Option<usize>,
    direction: Option<Direction>,
    main: i32,
    secondary: bool,
    secondary_pos: i32,
}
fn adjacent(a: [i32; 4], b: [i32; 4], axis: usize) -> bool {
    let between = |v, min, max| v > min && v < max;
    between(a[axis], b[axis], b[axis] + b[axis + 2])
        || between(a[axis] + a[axis + 2], b[axis], b[axis] + b[axis + 2])
        || between(b[axis], a[axis], a[axis] + a[axis + 2])
        || between(b[axis] + b[axis + 2], a[axis], a[axis] + a[axis + 2])
}
impl Snapper {
    pub fn unsnap(&mut self) {
        self.to = None
    }
    pub fn step(
        &mut self,
        box_: &mut HudBox,
        size: [i32; 2],
        delta: [i32; 2],
        viewport: [i32; 2],
        auto: bool,
        border: i32,
        range: i32,
        peers: &[(usize, [i32; 4])],
    ) {
        let rect = |b: HudBox| {
            let p = b.render(viewport, size);
            [p[0], p[1], size[0], size[1]]
        };
        if range == 0 {
            box_.move_by(delta, auto, border, viewport, size);
            return;
        }
        if let Some(other) = self
            .to
            .and_then(|id| peers.iter().find(|(i, _)| *i == id).map(|(_, r)| *r))
        {
            let horizontal = matches!(self.direction, Some(Direction::Right | Direction::Left));
            let (main_axis, second_axis) = if horizontal { (0, 1) } else { (1, 0) };
            if self.secondary {
                self.secondary_pos += delta[second_axis]
            } else {
                let mut d = [0, 0];
                d[second_axis] = delta[second_axis];
                box_.move_by(d, auto, border, viewport, size)
            }
            self.main += delta[main_axis];
            let current = rect(*box_);
            if !adjacent(current, other, second_axis) {
                self.unsnap()
            } else if !self.secondary {
                let end_dist = other[second_axis] + other[second_axis + 2]
                    - current[second_axis]
                    - current[second_axis + 2];
                let start_dist = current[second_axis] - other[second_axis];
                let snap = if delta[second_axis] > 0 && end_dist > 0 && end_dist < range {
                    Some(other[second_axis] + other[second_axis + 2] - size[second_axis])
                } else if delta[second_axis] < 0 && start_dist > 0 && start_dist < range {
                    Some(other[second_axis])
                } else {
                    None
                };
                if let Some(v) = snap {
                    let mut p = [current[0], current[1]];
                    p[second_axis] = v;
                    box_.set_pos(p, auto, viewport, size);
                    self.secondary = true;
                    self.secondary_pos = 0;
                }
            }
            if self.main.abs_diff(0) > range as u32 * 5 {
                self.unsnap()
            } else if self.secondary_pos.abs_diff(0) > range as u32 * 5 {
                self.secondary = false
            }
            return;
        }
        self.to = None;
        box_.move_by(delta, auto, border, viewport, size);
        let current = rect(*box_);
        let direction = if delta[0] > 0 {
            Direction::Right
        } else if delta[0] < 0 {
            Direction::Left
        } else if delta[1] > 0 {
            Direction::Top
        } else if delta[1] < 0 {
            Direction::Bottom
        } else {
            return;
        };
        let horizontal = matches!(direction, Direction::Left | Direction::Right);
        let candidate = peers
            .iter()
            .filter_map(|(id, other)| {
                if !adjacent(current, *other, if horizontal { 1 } else { 0 }) {
                    return None;
                }
                let dist = match direction {
                    Direction::Right => other[0] - current[0] - current[2],
                    Direction::Left => current[0] - other[0] - other[2],
                    Direction::Top => other[1] - current[1] - current[3],
                    Direction::Bottom => current[1] - other[1] - other[3],
                };
                (dist > 0 && dist <= range).then_some((*id, *other, dist))
            })
            .min_by_key(|(_, _, dist)| *dist);
        if let Some((id, other, _)) = candidate {
            let p = match direction {
                Direction::Right => [other[0] - size[0], current[1]],
                Direction::Left => [other[0] + other[2], current[1]],
                Direction::Top => [current[0], other[1] - size[1]],
                Direction::Bottom => [current[0], other[1] + other[3]],
            };
            box_.set_pos(p, auto, viewport, size);
            self.to = Some(id);
            self.direction = Some(direction);
            self.main = 0;
            self.secondary = false;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nearest_snap_then_main_hysteresis() {
        let mut b = HudBox {
            x: 10,
            y: 12,
            ..Default::default()
        };
        let mut s = Snapper::default();
        let peers = [(1, [35, 10, 10, 20]), (2, [38, 10, 10, 20])];
        s.step(&mut b, [10, 10], [6, 0], [100, 100], false, 0, 10, &peers);
        assert_eq!(b.render([100, 100], [10, 10]), [25, 12]);
        s.step(&mut b, [10, 10], [20, 0], [100, 100], false, 0, 10, &peers);
        assert_eq!(b.x, 25);
        s.step(&mut b, [10, 10], [31, 0], [100, 100], false, 0, 10, &peers);
        assert_eq!(b.x, 25);
        s.step(&mut b, [10, 10], [1, 0], [100, 100], false, 0, 10, &peers);
        // The next unsnapped move immediately acquires the second nearby element.
        assert_eq!(b.x, 28);
    }
    #[test]
    fn exact_overlap_boundaries_do_not_snap_source_strict_comparisons() {
        let mut b = HudBox {
            x: 10,
            y: 10,
            ..Default::default()
        };
        Snapper::default().step(
            &mut b,
            [10, 10],
            [6, 0],
            [100, 100],
            false,
            0,
            10,
            &[(1, [35, 10, 10, 10])],
        );
        assert_eq!(b.x, 16);
    }
}
