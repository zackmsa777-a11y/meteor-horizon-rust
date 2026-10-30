// SPDX-License-Identifier: GPL-3.0-only AND MIT
// Meteor HighJump policy: Copyright (c) Meteor Development, GPL-3.0-only.
// Native movement sequencing derived from Horizon b76ae35, under its MIT license.
// The corresponding Horizon LICENSE is retained by the standard source stage.
use super::{MotionWorld, PlayerMotion, MotionEvent, LivingMotionWorld,
    movement_support, RegistryMovementWorld, living_motion};
use horizon_core::geometry::{Vec3, trig};
use horizon_world::attributes::live::Handle;
use meteor_client::modules::high_jump::Settings;
use std::{cell::RefCell, ops::ControlFlow};

#[derive(Clone)]
struct Snapshot { owner: Handle, settings: Settings }
thread_local! { static POLICY: RefCell<Option<Snapshot>> = const { RefCell::new(None) }; }

/// Game publishes an owned native attribute identity, never a movable player address.
/// No Game/Runtime borrow or callback survives this function.
pub fn publish(owner: Handle, settings: Settings) -> Result<(), String> {
    clear();
    settings.validate()?;
    POLICY.with(|policy| *policy.borrow_mut()=Some(Snapshot { owner, settings }));
    Ok(())
}
pub fn clear() { POLICY.with(|policy| *policy.borrow_mut()=None); }

/// Check the actual MotionWorld player on each concrete jump/factor invocation.
/// A missing/unrealized/replaced attribute owner fails closed to native behavior.
fn selected(player: &PlayerMotion) -> Option<Settings> {
    let key=player.attributes.key("minecraft:jump_strength")?;
    let owner=player.attributes.lookup(Some(&key))?;
    selected_owner(&owner)
}
fn selected_owner(owner: &Handle) -> Option<Settings> {
    POLICY.with(|policy| policy.borrow().as_ref()
        .filter(|snapshot| &snapshot.owner==owner)
        .map(|snapshot|snapshot.settings.clone()))
}

/// Concrete cancellation replaces only the active local player's factor query.
/// Native lookup, then source f32 multiplier narrowing; jump boost remains additive.
pub fn block_factor(world: &mut MotionWorld<'_, '_, '_>) -> ControlFlow<crate::Result<f32>> {
    let Some(settings)=selected(world.player) else {return ControlFlow::Continue(());};
    let result=(|| -> crate::Result<f32> {
        let location=world.location();
        let original=movement_support::block_jump_factor(&mut RegistryMovementWorld {
            properties:&world.resources.blocks, world,
        },location)?;
        Ok(settings.block_jump_factor(original))
    })();
    ControlFlow::Break(result)
}

/// Concrete wrapper cancellation preserves both native tails exactly once.
/// The generic living_motion function and all other actors are unchanged.
pub fn jump(world: &mut MotionWorld<'_, '_, '_>) -> ControlFlow<crate::Result<()>> {
    if selected(world.player).is_none() { return ControlFlow::Continue(()); }
    ControlFlow::Break(active_jump(world))
}
fn active_jump(world: &mut MotionWorld<'_, '_, '_>) -> crate::Result<()> {
    // Queries strength, our block-factor hook, then additive boost once each.
    let power=living_motion::jump_power(world,1.0)?;
    // Source LivingEntityMixin changes the native 1e-5 threshold to -1.
    // Keep its NaN comparison semantics, including zero/tiny positive powers.
    if !(power <= -1.0f32) {
        let mut velocity=LivingMotionWorld::velocity(world);
        velocity.y=if power.is_nan() || velocity.y.is_nan() { f64::NAN }
            else { f64::from(power).max(velocity.y) };
        LivingMotionWorld::set_velocity(world,velocity)?;
        if LivingMotionWorld::sprinting(world)? {
            let angle=f64::from(LivingMotionWorld::yaw(world)? * trig::DEG_TO_RAD);
            living_motion::add_delta_movement(world,Vec3::new(
                f64::from(-trig::sin(angle))*0.2,0.0,f64::from(trig::cos(angle))*0.2))?;
        }
        LivingMotionWorld::mark_needs_sync(world);
    }
    // Native MotionWorld wrapper tail, once even if the inner power guard returns.
    world.player.events.push(MotionEvent::Health(crate::health::HealthEvent::Statistic {
        name:"jump", amount:1,
    }));
    world.player.cause_food_exhaustion(if world.player.sprinting {0.2}else{0.05});
    Ok(())
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    use horizon_world::attributes::{entity::Catalog,live::LiveAttributes};
    use std::sync::Arc;
    fn attrs() -> LiveAttributes<impl FnMut()->i32> {
        let bits=format!("{:016x}",0.42f64.to_bits());
        let catalog=Arc::new(Catalog::from_json(&serde_json::json!({
            "data_version":horizon_world::DATA_VERSION,
            "attributes":[{"id":"minecraft:jump_strength","default":bits,"syncable":true}],
            "entities":{"minecraft:player":{"minecraft:jump_strength":bits}},
        })).unwrap());
        LiveAttributes::new(catalog,"minecraft:player",&[7],||19).unwrap()
    }
    fn handle<G:FnMut()->i32>(attrs:&mut LiveAttributes<G>) -> Handle {
        let key=attrs.key("minecraft:jump_strength").unwrap();
        attrs.get_instance(Some(key)).unwrap().unwrap()
    }
    #[test]
    fn real_native_owner_excludes_same_index_and_survives_owner_move() {
        clear();
        let mut local=attrs();let mut foreign=attrs();
        let local_handle=handle(&mut local);let foreign_handle=handle(&mut foreign);
        assert_eq!(local_handle.index(),foreign_handle.index());
        assert!(local_handle!=foreign_handle);
        publish(local_handle.clone(),Settings{jump_multiplier:2.}).unwrap();
        assert_eq!(selected_owner(&local_handle).unwrap().jump_multiplier,2.);
        assert!(selected_owner(&foreign_handle).is_none());
        let moved=Box::new(local);
        let key=moved.key("minecraft:jump_strength").unwrap();
        assert!(selected_owner(&moved.lookup(Some(&key)).unwrap()).is_some());
        clear();assert!(selected_owner(&local_handle).is_none());
    }
    #[test]
    fn policy_is_thread_local_and_invalid_publication_clears_old_snapshot() {
        clear();let mut local=attrs();let owner=handle(&mut local);
        publish(owner.clone(),Settings::default()).unwrap();
        let other=owner.clone();
        assert!(std::thread::spawn(move||selected_owner(&other).is_none()).join().unwrap());
        assert!(publish(owner.clone(),Settings{jump_multiplier:f64::NAN}).is_err());
        assert!(selected_owner(&owner).is_none());
    }
}
