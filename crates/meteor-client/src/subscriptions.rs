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
