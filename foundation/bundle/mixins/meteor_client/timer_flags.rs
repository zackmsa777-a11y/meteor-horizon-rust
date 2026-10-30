// Solstice Timer switch, adapted by Meteor Horizon: client-thread scoped.
// Server clocks on other threads must not inherit a native client's multiplier.
use std::cell::Cell;
thread_local! {
    static MULT:Cell<f32> = const {Cell::new(1.)};
    static BUSY:Cell<bool> = const {Cell::new(false)};
}
pub fn set(value: f32) {
    MULT.with(|slot| {
        slot.set(if value.is_finite() && value > 0. {
            value.min(10.)
        } else {
            1.
        })
    });
}
pub fn multiplier() -> f32 {
    MULT.with(Cell::get)
}
pub struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        BUSY.with(|busy| busy.set(false));
    }
}
pub fn enter() -> Option<Guard> {
    BUSY.with(|busy| {
        if busy.replace(true) {
            None
        } else {
            Some(Guard)
        }
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiplier_and_recursion_guard_are_thread_local() {
        set(2.);
        let guard = enter().unwrap();
        assert!(enter().is_none());
        assert_eq!(
            std::thread::spawn(|| {
                assert!(enter().is_some());
                multiplier()
            })
            .join()
            .unwrap(),
            1.
        );
        drop(guard);
        assert!(enter().is_some());
        set(1.);
    }
    #[test]
    fn invalid_values_cannot_panic_duration_scaling() {
        for value in [f32::NAN, f32::INFINITY, 0., -1.] {
            set(value);
            assert_eq!(multiplier(), 1.);
        }
        set(1.);
    }
}
