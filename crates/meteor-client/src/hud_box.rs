// SPDX-License-Identifier: GPL-3.0-only
// Source-derived Meteor HudBox/HudElement geometry, Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum XAnchor {
    #[default]
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum YAnchor {
    #[default]
    Top,
    Center,
    Bottom,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HudBox {
    pub x_anchor: XAnchor,
    pub y_anchor: YAnchor,
    pub x: i32,
    pub y: i32,
}
impl HudBox {
    /// Source works in physical framebuffer pixels, with integer division at each half.
    pub fn render(self, viewport: [i32; 2], size: [i32; 2]) -> [i32; 2] {
        [
            match self.x_anchor {
                XAnchor::Left => self.x,
                XAnchor::Center => viewport[0] / 2 - size[0] / 2 + self.x,
                XAnchor::Right => viewport[0] - size[0] + self.x,
            },
            match self.y_anchor {
                YAnchor::Top => self.y,
                YAnchor::Center => viewport[1] / 2 - size[1] / 2 + self.y,
                YAnchor::Bottom => viewport[1] - size[1] + self.y,
            },
        ]
    }
    pub fn set_x_anchor(&mut self, anchor: XAnchor, viewport: [i32; 2], size: [i32; 2]) {
        if self.x_anchor == anchor {
            return;
        }
        let x = self.render(viewport, size)[0];
        self.x = match anchor {
            XAnchor::Left => x,
            XAnchor::Center => x + size[0] / 2 - viewport[0] / 2,
            XAnchor::Right => x + size[0] - viewport[0],
        };
        self.x_anchor = anchor;
    }
    pub fn set_y_anchor(&mut self, anchor: YAnchor, viewport: [i32; 2], size: [i32; 2]) {
        if self.y_anchor == anchor {
            return;
        }
        let y = self.render(viewport, size)[1];
        self.y = match anchor {
            YAnchor::Top => y,
            YAnchor::Center => y + size[1] / 2 - viewport[1] / 2,
            YAnchor::Bottom => y + size[1] - viewport[1],
        };
        self.y_anchor = anchor;
    }
    pub fn update_anchors(&mut self, viewport: [i32; 2], size: [i32; 2]) {
        let [x, y] = self.render(viewport, size);
        let left = x as f64 <= viewport[0] as f64 / 3.;
        let right = (x + size[0]) as f64 >= viewport[0] as f64 / 3. * 2.;
        let top = y as f64 <= viewport[1] as f64 / 3.;
        let bottom = (y + size[1]) as f64 >= viewport[1] as f64 / 3. * 2.;
        self.set_x_anchor(
            if left == right {
                XAnchor::Center
            } else if left {
                XAnchor::Left
            } else {
                XAnchor::Right
            },
            viewport,
            size,
        );
        self.set_y_anchor(
            if top == bottom {
                YAnchor::Center
            } else if top {
                YAnchor::Top
            } else {
                YAnchor::Bottom
            },
            viewport,
            size,
        );
    }
    /// HudElement.setPos differs from move: setPos has no border clamp.
    pub fn set_pos(&mut self, pos: [i32; 2], auto: bool, viewport: [i32; 2], size: [i32; 2]) {
        if auto {
            self.x = pos[0];
            self.y = pos[1];
            self.x_anchor = XAnchor::Left;
            self.y_anchor = YAnchor::Top;
            self.update_anchors(viewport, size)
        } else {
            let old = self.render(viewport, size);
            self.x += pos[0] - old[0];
            self.y += pos[1] - old[1];
        }
    }
    pub fn move_by(
        &mut self,
        delta: [i32; 2],
        auto: bool,
        border: i32,
        viewport: [i32; 2],
        size: [i32; 2],
    ) {
        self.x += delta[0];
        self.y += delta[1];
        if auto {
            self.update_anchors(viewport, size)
        }
        // Preserve source's positive right/bottom border bound exactly.
        if self.x_anchor == XAnchor::Left && self.x < border
            || self.x_anchor == XAnchor::Right && self.x > border
        {
            self.x = border
        }
        if self.y_anchor == YAnchor::Top && self.y < border
            || self.y_anchor == YAnchor::Bottom && self.y > border
        {
            self.y = border
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn odd_center_preserves_source_integer_halves() {
        let b = HudBox {
            x_anchor: XAnchor::Center,
            y_anchor: YAnchor::Center,
            x: 2,
            y: -3,
        };
        assert_eq!(b.render([101, 91], [31, 21]), [37, 32]);
    }
    #[test]
    fn reanchor_preserves_pixels_and_resize_responds() {
        let mut b = HudBox {
            x: 80,
            y: 70,
            ..Default::default()
        };
        b.update_anchors([101, 91], [20, 20]);
        assert_eq!(b.x_anchor, XAnchor::Right);
        assert_eq!(b.y_anchor, YAnchor::Bottom);
        assert_eq!(b.render([101, 91], [20, 20]), [80, 70]);
        assert_eq!(b.render([201, 191], [20, 20]), [180, 170]);
    }
    #[test]
    fn explicit_anchor_setpos_and_border_differ() {
        let mut b = HudBox {
            x_anchor: XAnchor::Right,
            y_anchor: YAnchor::Bottom,
            x: -4,
            y: -4,
        };
        b.set_pos([10, 11], false, [100, 100], [20, 20]);
        assert_eq!(b.x_anchor, XAnchor::Right);
        assert_eq!(b.render([100, 100], [20, 20]), [10, 11]);
        b.move_by([100, 100], false, 4, [100, 100], [20, 20]);
        assert_eq!(b.render([100, 100], [20, 20]), [84, 84]);
    }
    #[test]
    fn spans_both_thirds_anchors_center() {
        let mut b = HudBox {
            x: 20,
            y: 20,
            ..Default::default()
        };
        b.update_anchors([90, 90], [50, 50]);
        assert_eq!(b.x_anchor, XAnchor::Center);
        assert_eq!(b.y_anchor, YAnchor::Center);
        assert_eq!(b.render([90, 90], [50, 50]), [20, 20]);
    }
}
