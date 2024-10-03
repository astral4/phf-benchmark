#![cfg_attr(not(test), no_std)]

use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use hashbrown::HashSet;

pub mod hash;

pub const FIXED_SEED: u64 = 42;

pub trait Seedable: Hasher {
    type Seed;

    fn new_with_seed(seed: &Self::Seed) -> Self;
}

pub trait PhfMap {
    type Key;
    type Value;

    fn get_entry<T>(&self, key: &T) -> Option<(&Self::Key, &Self::Value)>
    where
        T: Eq + Hash + ?Sized,
        Self::Key: Borrow<T>;
}

pub fn has_duplicates<T: Eq + Hash>(items: &[T]) -> bool {
    let mut set = HashSet::with_capacity(items.len());

    for item in items {
        if !set.insert(item) {
            return true;
        }
    }

    false
}
