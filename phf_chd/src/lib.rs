#![cfg_attr(not(test), no_std)]

extern crate alloc;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use core::hash::{Hash, Hasher};
use num_traits::bounds::UpperBounded;
use num_traits::{AsPrimitive, Unsigned, WrappingAdd, WrappingMul, Zero};
use phf_shared::hash::AHasher;
use phf_shared::{has_duplicates, PhfMap, Seedable};
use rand::distributions::{Distribution, Standard};
use usize_cast::IntoUsize;

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

impl ChdHasher for AHasher {
    type Hash = u16;

    #[allow(clippy::cast_possible_truncation)]
    fn finish_triple(&self) -> Hashes<Self> {
        let output = self.finish();
        ((output >> 32) as u16, (output >> 16) as u16, output as u16)
    }
}

pub struct MapGenerator<K, V, H: ChdHasher> {
    seed: H::Seed,
    disps: Vec<(H::Hash, H::Hash)>,
    indices: Vec<usize>,
    entries: Vec<(K, V)>,
}

impl<K, V, H, I> From<I> for MapGenerator<K, V, H>
where
    K: Eq + Hash,
    H: ChdHasher,
    I: Iterator<Item = (K, V)>,
    Standard: Distribution<H::Seed>,
    usize: AsPrimitive<H::Hash>,
{
    fn from(value: I) -> Self {
        let entries: Vec<_> = value.collect();

        assert!(
            entries.len() <= H::Hash::max_value().into_usize(),
            "cannot have more entries than possible hash values"
        );

        let keys: Vec<_> = entries.iter().map(|entry| &entry.0).collect();

        assert!(!has_duplicates(&keys), "duplicate key present");

        let (seed, state) = generate::generate::<_, H>(&keys);

        Self {
            seed,
            disps: state.displacements,
            indices: state.indices,
            entries,
        }
    }
}

impl<K, V, H> Display for MapGenerator<K, V, H>
where
    K: Debug,
    V: Debug,
    H: ChdHasher,
    H::Seed: Debug,
    H::Hash: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_fmt(format_args!(
            "::phf_chd::Map {{\
                seed: {:?},\
                disps: &{:?},\
                entries: &[",
            &self.seed, &self.disps
        ))?;

        for &index in &self.indices {
            f.write_fmt(format_args!("{:?},", &self.entries[index]))?;
        }

        f.write_str("]}")
    }
}

pub struct Map<K: 'static, V: 'static, H: ChdHasher> {
    pub seed: H::Seed,
    pub disps: &'static [(H::Hash, H::Hash)],
    pub entries: &'static [(K, V)],
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

        let hashes = generate::hash::<_, H>(key, &self.seed);
        let (d1, d2) = self.disps[hashes.0.into_usize() % self.disps.len()];
        let index =
            generate::displace(hashes.1, hashes.2, d1, d2).into_usize() % self.entries.len();
        let entry = &self.entries[index];

        if entry.0.borrow() == key {
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }
}
