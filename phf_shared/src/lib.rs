#![warn(clippy::pedantic, future_incompatible, unused)]
#![deny(
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    non_ascii_idents,
    nonstandard_style,
    noop_method_call,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_results,
    unused_tuple_struct_fields
)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;
use ahash::RandomState;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::iter::zip;
use indexmap::IndexSet;

pub mod hash;

pub const FIXED_SEED: u64 = 42;

pub trait Seedable: Hasher {
    type Seed: Copy;

    fn new_with_seed(seed: Self::Seed) -> Self;
}

pub trait PhfMap {
    type Key;
    type Value;

    fn get_entry<T>(&self, key: &T) -> Option<(&Self::Key, &Self::Value)>
    where
        T: Eq + Hash + ?Sized,
        Self::Key: Borrow<T>;
}

pub struct MapBuilder<K, V> {
    keys: IndexSet<K, RandomState>,
    values: Vec<V>,
}

impl<K, V> MapBuilder<K, V>
where
    K: Eq + Hash,
{
    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            keys: IndexSet::with_capacity_and_hasher(capacity, RandomState::new()), // HashSet::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn entry(&mut self, key: K, value: V) -> &mut Self {
        if !self.keys.insert(key) {
            panic!("duplicate key inserted");
        }
        self.values.push(value);
        self
    }

    pub fn finish(self) -> Vec<(K, V)> {
        zip(self.keys, self.values).collect()
    }
}
