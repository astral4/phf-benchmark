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
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::hash::Hash;
use num_traits::bounds::UpperBounded;
use num_traits::{AsPrimitive, Unsigned, WrappingAdd, WrappingMul, Zero};
use phf_shared::{PhfMap, Seedable};
use rand::distributions::{Distribution, Standard};
use usize_cast::IntoUsize;

#[cfg(test)]
use ahash as _;

mod generate;

pub trait ChdHasher: Seedable {
    type Hash: 'static
        + UpperBounded
        + Unsigned
        + IntoUsize
        + Zero
        + Copy
        + WrappingMul
        + WrappingAdd;

    fn finish_triple(&self) -> Hashes<Self>;
}

pub type Hashes<H> = (
    <H as ChdHasher>::Hash,
    <H as ChdHasher>::Hash,
    <H as ChdHasher>::Hash,
);

pub struct Map<K: 'static, V: 'static, H: ChdHasher> {
    pub seed: H::Seed,
    pub disps: &'static [(H::Hash, H::Hash)],
    pub indices: &'static [usize],
    pub entries: &'static [(K, V)],
}

impl<K, V, H> Map<K, V, H>
where
    K: Hash,
    H: ChdHasher,
    Standard: Distribution<H::Seed>,
    usize: AsPrimitive<H::Hash>,
{
    pub fn from_iter<I: Iterator<Item = (K, V)>>(entries: I) -> Self {
        let entries: Vec<_> = entries.collect();

        assert!(
            entries.len() <= H::Hash::max_value().into_usize(),
            "cannot have more entries than possible hash values"
        );

        let keys: Vec<_> = entries.iter().map(|entry| &entry.0).collect();
        let (seed, state) = generate::generate::<_, H>(&keys);

        Self {
            seed,
            disps: state.displacements.leak(),
            indices: state.indices.leak(),
            entries: entries.leak(),
        }
    }
}

impl<K, V, H> PhfMap for Map<K, V, H>
where
    H: ChdHasher,
{
    type Key = K;
    type Value = V;

    fn get_entry<T>(&self, key: &T) -> Option<(&Self::Key, &Self::Value)>
    where
        T: Eq + Hash + ?Sized,
        Self::Key: Borrow<T>,
    {
        if self.disps.is_empty() {
            return None;
        }

        let hashes = generate::hash::<_, H>(key, self.seed);
        let (d1, d2) = self.disps[hashes.0.into_usize() % self.disps.len()];
        let indices_index =
            generate::displace(hashes.1, hashes.2, d1, d2).into_usize() % self.indices.len();
        let index = self.indices[indices_index];
        let entry = &self.entries[index];

        if entry.0.borrow() == key {
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }
}
