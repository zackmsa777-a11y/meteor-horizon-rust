// SPDX-License-Identifier: GPL-3.0-only
// MeteorStarscript/Utils.getPlayerSpeed/HorizontalDirection/TickRate native logic.
// Copyright (c) Meteor Development.
// Upstream TickRate credits KAMI Blue's LagCompensator:
// https://github.com/S-B99/kamiblue/blob/feature/master/src/main/java/me/zeroeightsix/kami/util/LagCompensator.java
pub fn speed(
    current: [f64; 3],
    previous: [f64; 3],
    timer_active: bool,
    timer_multiplier: f64,
) -> f64 {
    let factor = if timer_active { timer_multiplier } else { 1.0 } * 20.0;
    ((current[0] - previous[0]) * factor).hypot((current[2] - previous[2]) * factor)
}
pub fn angle(mut angle: f32) -> f32 {
    angle %= 360.;
    if angle < 0. {
        angle += 360.;
    }
    if angle > 180. {
        angle -= 360.;
    }
    angle
}
pub fn direction(yaw: f32) -> &'static str {
    let yaw = yaw.rem_euclid(360.);
    if !(22.5..337.5).contains(&yaw) {
        "South"
    } else if yaw < 67.5 {
        "South West"
    } else if yaw < 112.5 {
        "West"
    } else if yaw < 157.5 {
        "North West"
    } else if yaw < 202.5 {
        "North"
    } else if yaw < 247.5 {
        "North East"
    } else if yaw < 292.5 {
        "East"
    } else {
        "South East"
    }
}

/// The source estimates TPS from arrival intervals of ClientboundSetTimePacket, averaging
/// its last twenty positive samples. It intentionally reports20 during the join grace period.
#[derive(Clone, Debug)]
pub struct TickRate {
    joined_ms: u64,
    last_ms: u64,
    samples: [f32; 20],
    next: usize,
}
impl Default for TickRate {
    fn default() -> Self {
        Self::joined(0)
    }
}
impl TickRate {
    pub fn joined(now_ms: u64) -> Self {
        Self {
            joined_ms: now_ms,
            last_ms: now_ms,
            samples: [0.; 20],
            next: 0,
        }
    }
    pub fn packet(&mut self, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.last_ms) as f32 / 1000.;
        self.samples[self.next] = (20. / elapsed).clamp(0., 20.);
        self.next = (self.next + 1) % 20;
        self.last_ms = now_ms;
    }
    pub fn rate(&self, now_ms: u64, in_world: bool) -> f32 {
        if !in_world {
            return 0.;
        }
        if now_ms.saturating_sub(self.joined_ms) < 4000 {
            return 20.;
        }
        let mut sum = 0.;
        let mut count = 0;
        for sample in self.samples {
            if sample > 0. {
                sum += sample;
                count += 1;
            }
        }
        // Source returns NaN when no sample exists after its grace period; never invent20.
        sum / count as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn horizontal_speed_uses_displacement_and_only_active_timer_scaling() {
        assert_eq!(speed([3., 99., 4.], [0., 0., 0.], false, 2.), 100.);
        assert_eq!(speed([3., 99., 4.], [0., 0., 0.], true, 2.), 200.);
    }
    #[test]
    fn camera_angles_and_diagonal_boundaries_match_source() {
        assert_eq!(angle(-180.), 180.);
        assert_eq!(angle(541.), -179.);
        assert_eq!(direction(22.49), "South");
        assert_eq!(direction(22.5), "South West");
        assert_eq!(direction(-45.), "South East");
        assert_eq!(direction(337.5), "South");
    }
    #[test]
    fn tps_samples_packets_instead_of_client_frames() {
        let mut t = TickRate::joined(1000);
        assert_eq!(t.rate(4999, true), 20.);
        assert!(t.rate(5000, true).is_nan());
        t.packet(6000); // Five seconds ->4TPS.
        t.packet(7000); // One second ->20TPS.
        assert_eq!(t.rate(7000, true), 12.);
        for at in 8..=27 {
            t.packet(at * 1000);
        }
        assert_eq!(t.rate(27000, true), 20.);
        assert_eq!(t.rate(27000, false), 0.);
        t = TickRate::joined(30000);
        assert_eq!(t.rate(30000, true), 20.);
    }
}
