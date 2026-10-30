// SPDX-License-Identifier: GPL-3.0-only
// Module.toggle/world lifecycle + Orbit equal-priority listener ordering.
// Copyright (c) Meteor Development.
//! Runtime subscriptions are not persistent configuration. Equal-priority listeners run in
//! subscription order; disabling and re-enabling appends a fresh activation at the end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subscription {
    pub id: String,
    pub generation: u64,
}
#[derive(Clone, Debug, Default)]
pub struct Subscriptions {
    entries: Vec<Subscription>,
    generation: u64,
}
impl Subscriptions {
    pub fn activate(&mut self, id: &str) {
        self.deactivate(id);
        self.generation = self.generation.wrapping_add(1);
        self.entries.push(Subscription {
            id: id.to_owned(),
            generation: self.generation,
        });
    }
    pub fn deactivate(&mut self, id: &str) {
        self.entries.retain(|entry| entry.id != id);
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn entries(&self) -> &[Subscription] {
        &self.entries
    }
    pub fn generation(&self, id: &str) -> Option<u64> {
        self.entries
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.generation)
    }
}

/// Implemented equal-priority TickEvent.Post subscribers. The order comes from
/// activation generations, never from this enum or the static module registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostTick {
    Step,
    ReverseStep,
    FastClimb,
    Spider,
    Parkour,
    Flight,
    AutoClicker,
    Fullbright,
}
impl PostTick {
    pub fn from_id(id: &str) -> Option<Self> {
        Some(match id {
            "step" => Self::Step,
            "reverse-step" => Self::ReverseStep,
            "fast-climb" => Self::FastClimb,
            "spider" => Self::Spider,
            "parkour" => Self::Parkour,
            "flight" => Self::Flight,
            "auto-clicker" => Self::AutoClicker,
            "fullbright" => Self::Fullbright,
            _ => return None,
        })
    }
}
/// Snapshot the event bus before callbacks can mutate subscriptions.
pub fn ordered_post_tick(subscriptions: &[Subscription]) -> Vec<PostTick> {
    subscriptions
        .iter()
        .filter_map(|s| PostTick::from_id(&s.id))
        .collect()
}
