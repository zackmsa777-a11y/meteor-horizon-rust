// Solstice No Slow switch, adapted by Meteor Horizon: client-thread scoped.
use std::cell::Cell;
thread_local! {static NO_SLOW:Cell<bool> = const {Cell::new(false)};}
pub fn set(value: bool) {
    NO_SLOW.with(|slot| slot.set(value));
}
pub fn enabled() -> bool {
    NO_SLOW.with(Cell::get)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn another_threads_movement_keeps_native_slowdown() {
        set(true);
        assert!(!std::thread::spawn(enabled).join().unwrap());
        set(false);
        assert!(!enabled());
    }
}
