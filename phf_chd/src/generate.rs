use alloc::vec;
use alloc::vec::Vec;
use core::hash::Hash;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, Zero};
use phf_shared::FIXED_SEED;
use rand::distributions::{Distribution, Standard};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use usize_cast::IntoUsize;

use super::{ChdHasher, Hashes};

const LAMBDA: usize = 5;

pub(crate) struct MapState<H: ChdHasher> {
    pub(crate) displacements: Vec<(H::Hash, H::Hash)>,
    pub(crate) indices: Vec<usize>,
}

struct Bucket {
    index: usize,
    keys: Vec<usize>,
}

impl Bucket {
    fn new(index: usize) -> Self {
        Self {
            index,
            keys: Vec::new(),
        }
    }
}

pub(crate) fn generate<T, H>(entries: &[T]) -> (H::Seed, MapState<H>)
where
    T: Hash,
    H: ChdHasher,
    Standard: Distribution<H::Seed>,
    usize: AsPrimitive<H::Hash>,
{
    SmallRng::seed_from_u64(FIXED_SEED)
        .sample_iter(Standard)
        .find_map(|seed| {
            let hashes: Vec<_> = entries
                .iter()
                .map(|entry| hash::<_, H>(entry, seed))
                .collect();
            try_generate::<H>(&hashes).map(|s| (seed, s))
        })
        .expect("failed to obtain PHF")
}

fn try_generate<H>(hashes: &[Hashes<H>]) -> Option<MapState<H>>
where
    H: ChdHasher,
    usize: AsPrimitive<H::Hash>,
{
    let table_len = hashes.len();
    let num_buckets = (table_len + LAMBDA - 1) / LAMBDA;

    let mut buckets: Vec<_> = (0..num_buckets).map(Bucket::new).collect();

    for (i, hash) in hashes.iter().enumerate() {
        buckets[hash.0.into_usize() % num_buckets].keys.push(i);
    }
    buckets.sort_by(|a, b| Ord::cmp(&a.keys.len(), &b.keys.len()).reverse());

    let mut displacements = vec![(H::Hash::zero(), H::Hash::zero()); num_buckets];
    let mut map = vec![None; table_len];
    let mut try_map = vec![0u64; table_len];
    let mut generation = 0;
    let mut values_to_add = Vec::with_capacity(LAMBDA);

    'buckets: for bucket in &buckets {
        for d1 in 0..table_len {
            'disps: for d2 in 0..table_len {
                let (d1, d2) = (d1.as_(), d2.as_());
                values_to_add.clear();
                generation += 1;

                for &key in &bucket.keys {
                    let index =
                        displace(hashes[key].1, hashes[key].2, d1, d2).into_usize() % table_len;

                    if map[index].is_some() || try_map[index] == generation {
                        continue 'disps;
                    }

                    try_map[index] = generation;
                    values_to_add.push((index, key));
                }

                displacements[bucket.index] = (d1, d2);
                for &(index, key) in &values_to_add {
                    map[index] = Some(key);
                }
                continue 'buckets;
            }
        }
        return None;
    }

    Some(MapState {
        displacements,
        indices: map.into_iter().map(Option::unwrap).collect(),
    })
}

pub(crate) fn hash<T: Hash, H: ChdHasher>(x: T, seed: H::Seed) -> Hashes<H> {
    let mut hasher = H::new_with_seed(seed);
    x.hash(&mut hasher);
    hasher.finish_triple()
}

pub(crate) fn displace<T: WrappingMul + WrappingAdd>(f1: T, f2: T, d1: T, d2: T) -> T {
    f1.wrapping_mul(&d1).wrapping_add(&f2).wrapping_add(&d2)
}
