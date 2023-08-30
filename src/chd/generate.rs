use core::hash::Hash;
use num_traits::AsPrimitive;
use rand::distributions::{Distribution, Standard};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::{ChdHasher, Hashes};

const FIXED_SEED: u64 = 42;
const LAMBDA: usize = 5;

pub(super) struct MapState {
    pub(super) displacements: Vec<(usize, usize)>,
    pub(super) map: Vec<usize>,
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

pub(super) fn generate<T, H>(entries: &[T]) -> (H::Seed, MapState)
where
    T: Hash,
    H: ChdHasher,
    Standard: Distribution<H::Seed>,
{
    SmallRng::seed_from_u64(FIXED_SEED)
        .sample_iter(Standard)
        .find_map(|seed| {
            let hashes: Box<_> = entries
                .iter()
                .map(|entry| hash::<_, H>(entry, seed))
                .collect();
            try_generate::<H>(&hashes).map(|s| (seed, s))
        })
        .expect("failed to obtain PHF")
}

fn try_generate<H: ChdHasher>(hashes: &[Hashes<H>]) -> Option<MapState> {
    let table_len = hashes.len();
    let num_buckets = (table_len + LAMBDA - 1) / LAMBDA;

    let mut buckets: Vec<_> = (0..num_buckets).map(Bucket::new).collect();

    for (i, hash) in hashes.iter().enumerate() {
        buckets[hash.0.as_() % num_buckets].keys.push(i);
    }
    buckets.sort_by(|a, b| Ord::cmp(&a.keys.len(), &b.keys.len()).reverse());

    let mut map = vec![None; table_len];
    let mut displacements = vec![(0, 0); num_buckets];
    let mut try_map = vec![0u64; table_len];
    let mut generation = 0;

    'buckets: for bucket in &buckets {
        for d1 in 0..table_len {
            'disps: for d2 in 0..table_len {
                let mut values_to_add = Vec::with_capacity(LAMBDA);
                generation += 1;

                for &key in &bucket.keys {
                    let index =
                        displace(hashes[key].1.as_(), hashes[key].2.as_(), d1, d2) % table_len;
                    // maybe d1.into(), d2.into() instead?

                    if map[index].is_some() || try_map[index] == generation {
                        continue 'disps;
                    }

                    try_map[index] = generation;
                    values_to_add.push((index, key));
                }

                displacements[bucket.index] = (d1, d2);
                for (index, key) in values_to_add {
                    map[index] = Some(key);
                }
                continue 'buckets;
            }
        }
        return None;
    }

    Some(MapState {
        displacements,
        map: map.into_iter().map(Option::unwrap).collect(),
    })
}

/*
fn hash<T: Hash + ?Sized, H: ChdHasher>(x: &T, seed: H::Seed) -> Hashes<H> {
    let mut hasher = H::new_with_seed(seed);
    x.hash(&mut hasher);
    hasher.finish_triple()
}
*/

pub(super) fn hash<T: Hash, H: ChdHasher>(x: T, seed: H::Seed) -> Hashes<H> {
    let mut hasher = H::new_with_seed(seed);
    x.hash(&mut hasher);
    hasher.finish_triple()
}

pub(super) fn displace(f1: usize, f2: usize, d1: usize, d2: usize) -> usize {
    f1.wrapping_mul(d1).wrapping_add(f2).wrapping_add(d2)
}
