// SPDX-License-Identifier: GPL-3.0-only
// DamageUtils.java crystal damage/exposure. Copyright (c) Meteor Development.
//! Client-side prediction only. This never damages an entity or changes equipment.

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

#[derive(Debug, PartialEq)]
pub enum ExposureError<E> {
    InvalidBounds,
    Ray(E),
}

/// Source's world-coordinate additive loop, not vanilla's normalized interpolation loop.
/// `blocked` must use BlockGetter.traverseBlocks, ignore blocks with explosion resistance
/// below 600, and clip remaining block collision shapes with the empty context. No fluids.
pub fn exposure<E>(
    source: [f64; 3],
    bounds: Bounds,
    mut blocked: impl FnMut([f64; 3], [f64; 3]) -> Result<bool, E>,
) -> Result<f32, ExposureError<E>> {
    let diff: [f64; 3] = std::array::from_fn(|i| bounds.max[i] - bounds.min[i]);
    // Real player boxes have positive finite dimensions. Java's zero-size case loops forever;
    // report invalid geometry instead of hanging the render/game thread.
    if diff.iter().any(|d| !d.is_finite() || *d <= 0.0)
        || bounds
            .min
            .iter()
            .chain(bounds.max.iter())
            .any(|v| !v.is_finite())
    {
        return Err(ExposureError::InvalidBounds);
    }
    let fractions: [f64; 3] = diff.map(|d| 1.0 / (d * 2.0 + 1.0));
    let offset = [
        (1.0 - (1.0 / fractions[0]).floor() * fractions[0]) * 0.5,
        0.0,
        (1.0 - (1.0 / fractions[2]).floor() * fractions[2]) * 0.5,
    ];
    let increment: [f64; 3] = std::array::from_fn(|i| fractions[i] * diff[i]);
    let start: [f64; 3] = std::array::from_fn(|i| bounds.min[i] + offset[i]);
    let end: [f64; 3] = std::array::from_fn(|i| bounds.max[i] + offset[i]);
    if (0..3).any(|i| start[i] + increment[i] <= start[i]) {
        return Err(ExposureError::InvalidBounds);
    }
    let (mut misses, mut hits) = (0u32, 0u32);
    let mut x = start[0];
    while x <= end[0] {
        let mut y = start[1];
        while y <= end[1] {
            let mut z = start[2];
            while z <= end[2] {
                if !blocked([x, y, z], source).map_err(ExposureError::Ray)? {
                    misses += 1;
                }
                hits += 1;
                z += increment[2];
            }
            y += increment[1];
        }
        x += increment[0];
    }
    Ok(misses as f32 / hits as f32)
}

#[derive(Clone, Copy, Debug)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}
#[derive(Clone, Copy, Debug)]
pub struct Mitigation {
    pub difficulty: Difficulty,
    pub scales_with_difficulty: bool,
    pub armor: f64,
    pub toughness: f64,
    pub resistance_amplifier: Option<i32>,
    /// Source manually sums positive Protection + 2*BlastProtection levels on ARMOR slots.
    pub protection: i32,
    pub bypasses_invulnerability: bool,
}

/// The source floors armor, then performs CombatRules arithmetic in float precision.
/// Its crystal source has no weapon: no armor-effectiveness enchantment callback applies.
pub fn reduce(mut damage: f32, mitigation: Mitigation) -> f32 {
    if mitigation.scales_with_difficulty {
        match mitigation.difficulty {
            Difficulty::Easy => damage = (damage / 2.0 + 1.0).min(damage),
            Difficulty::Hard => damage *= 1.5,
            _ => {}
        }
    }
    let armor = mitigation.armor.floor() as f32;
    let toughness = mitigation.toughness as f32;
    let divisor = 2.0 + toughness / 4.0;
    let effectiveness = (armor - damage / divisor).max(armor * 0.2).min(20.0) / 25.0;
    damage *= 1.0 - effectiveness;
    if let Some(amplifier) = mitigation.resistance_amplifier {
        damage *= 1.0 - amplifier.wrapping_add(1) as f32 * 0.2;
    }
    damage = damage.max(0.0);
    if !mitigation.bypasses_invulnerability {
        damage *= 1.0 - (mitigation.protection as f32).clamp(0.0, 20.0) / 25.0;
    }
    damage.max(0.0)
}

pub fn distance(position: [f64; 3], crystal: [f64; 3]) -> f64 {
    let d: [f64; 3] = std::array::from_fn(|i| position[i] - crystal[i]);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Pass the player's unshifted position even when SafeStep is probing a translated box.
/// Creative player exclusion belongs before the exposure queries in the host.
pub fn crystal_damage(
    position: [f64; 3],
    crystal: [f64; 3],
    exposure: f32,
    mitigation: Mitigation,
) -> f32 {
    let distance = distance(position, crystal);
    if distance > 12.0 {
        return 0.0;
    }
    let impact = (1.0 - distance / 12.0) * f64::from(exposure);
    let damage = ((impact * impact + impact) / 2.0 * 7.0 * 12.0 + 1.0) as i32 as f32;
    reduce(damage, mitigation)
}
