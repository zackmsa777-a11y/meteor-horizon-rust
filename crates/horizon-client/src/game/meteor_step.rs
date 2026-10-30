// SPDX-License-Identifier: GPL-3.0-only
// Step.java lifecycle and SafeStep native adapter. Copyright (c) Meteor Development.
use super::{Game, Result, view_vector};
use horizon_client::motion::PlayerMotion;
use meteor_client::{
    damage::Bounds,
    modules::step::{self, Settings, Snapshot},
};
use std::io;

#[derive(Default)]
pub(super) struct State {
    snapshot: Option<Snapshot>,
}
impl State {
    /// A new player/world has a different attribute instance; do not restore the old player's
    /// baseline into the replacement. Call during the bridge's world/player reset handling.
    pub(super) fn reset(&mut self) {
        self.snapshot = None;
    }
}

fn set_base(player: &mut PlayerMotion, height: f64) -> Result<()> {
    let key = player
        .attributes
        .key("minecraft:step_height")
        .ok_or("missing step-height attribute")?;
    let handle = player
        .attributes
        .get_instance(Some(key))
        .map_err(|e| io::Error::other(format!("{e:?}")))?
        .ok_or("missing step-height attribute instance")?;
    player
        .attributes
        .edit(&handle, |attribute, observer| {
            attribute.set_base_value(height, observer)
        })
        .map_err(|e| io::Error::other(format!("{e:?}")))?;
    Ok(())
}

impl Game<'_> {
    /// Call each Tick.Post, even when disabled (settings=None), so disable restores the base.
    /// Temporarily take State out of Bridge before calling, then put it back even on errors.
    pub(super) fn meteor_step_post(
        &mut self,
        state: &mut State,
        settings: Option<&Settings>,
    ) -> Result<()> {
        let Some(player) = self.player.as_mut() else {
            state.reset();
            return Ok(());
        };
        let Some(settings) = settings else {
            if let Some(snapshot) = state.snapshot {
                set_base(player, snapshot.restore_base())?;
                state.snapshot = None;
            }
            return Ok(());
        };
        settings.validate()?;
        if state.snapshot.is_none() {
            let effective = player.attribute("minecraft:step_height")?;
            // Native Horizon has no Baritone path-manager settings. The player attribute
            // lifecycle is implemented; toggling/restoring Baritone's step flag remains absent.
            state.snapshot = Some(Snapshot::capture(effective, false));
        }
        let snapshot = state.snapshot.unwrap();
        let aabb = player.bounds();
        let min = aabb.min_position();
        let max = aabb.max_position();
        let original = Bounds {
            min: [min.x, min.y, min.z],
            max: [max.x, max.y, max.z],
        };
        let health = player.health.health();
        let absorption = player.health.absorption();
        let shift = player.is_shift_key_down();
        let offset = step::input_offset(view_vector(self.yaw, self.pitch), player.move_vector);
        let max_safe = step::max_safe_height(settings, health, absorption, offset, |translation| {
            self.meteor_crystal_damage_at_box(Bounds {
                min: std::array::from_fn(|i| original.min[i] + translation[i]),
                max: std::array::from_fn(|i| original.max[i] + translation[i]),
            })
        });
        match max_safe {
            Ok(height) => {
                if let Some(player) = self.player.as_mut() {
                    set_base(player, snapshot.tick_base(settings, shift, height))?;
                }
                Ok(())
            }
            Err(error) => {
                // A failed scene query must not leave an earlier raised step-height active.
                if let Some(player) = self.player.as_mut() {
                    set_base(player, snapshot.restore_base())?;
                }
                let message = match error {
                    step::SafeHeightError::Probe(error) => {
                        format!("SafeStep crystal query failed: {error}")
                    }
                    step::SafeHeightError::SweepLimit => format!(
                        "SafeStep exceeds native sweep limit ({} intermediate heights)",
                        step::MAX_SAFE_SWEEP_STEPS
                    ),
                };
                Err(message.into())
            }
        }
    }
}
