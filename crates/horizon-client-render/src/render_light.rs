// SPDX-License-Identifier: GPL-3.0-only
// Render-only Fullbright luminance sampling. Copyright (c) Meteor Development.
//! Apply at raw brightness getter boundaries, not to world simulation or final lightmaps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Floors {
    pub block: u8,
    pub sky: u8,
}
impl Floors {
    pub fn new(block: u8, sky: u8) -> Self {
        Self {
            block: block.min(15),
            sky: sky.min(15),
        }
    }
    /// Stored light is sky<<4|block. Clamp each sample BEFORE smooth/weighted light blends.
    /// LOD stores channel coordinates (sky << 4 | block << 12). LOD only
    /// combines brightness with max, which commutes exactly with these floors.
    pub fn coords(self, coords: u16) -> u16 {
        let packed = ((coords >> 4) as u8 & 15) << 4 | ((coords >> 12) as u8 & 15);
        let packed = self.sample(packed);
        u16::from(packed & 0xf0) | (u16::from(packed & 15) << 12)
    }
    pub fn sample(self, packed: u8) -> u8 {
        (packed & 15).max(self.block) | ((packed >> 4).max(self.sky) << 4)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fullbright_channels_are_independent_and_sampling_precedes_averaging() {
        let block = Floors::new(8, 0);
        let sky = Floors::new(0, 9);
        assert_eq!(block.sample(0x35), 0x38);
        assert_eq!(sky.sample(0x35), 0x95);
        assert_eq!(block.sample(0xff), 0xff);
        assert_eq!(Floors::default().sample(0x35), 0x35);
        let before = (u16::from(block.sample(0) & 15) + u16::from(block.sample(15) & 15)) / 2;
        let after = block.sample((0 + 15) / 2) & 15;
        assert_eq!(before, 11);
        assert_eq!(after, 8); // proves post-blend clamping changes behavior
    }
}

#[cfg(test)]
mod lod_floor_tests {
    use super::*;
    #[test]
    fn fullbright_lod_channel_maxima_commute_with_floors() {
        for floor in 0..=15 {
            for a in 0..=15u8 {
                for b in 0..=15u8 {
                    let f = Floors::new(floor, 0);
                    assert_eq!(f.sample(a.max(b)), f.sample(a).max(f.sample(b)));
                    assert_eq!(f.coords(u16::from(a) << 12), u16::from(a.max(floor)) << 12);
                }
            }
        }
    }
}
