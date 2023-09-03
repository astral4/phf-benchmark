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

use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use hashbrown::HashSet;

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

pub fn has_duplicates<T: Eq + Hash>(items: &[T]) -> bool {
    let mut set = HashSet::with_capacity(items.len());

    for item in items {
        if !set.insert(item) {
            return false;
        }
    }

    true
}
