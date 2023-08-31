use core::hash::Hash;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, Zero};
use rand::distributions::{Distribution, Standard};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::{ChdHasher, Hashes};

const FIXED_SEED: u64 = 42;
const LAMBDA: usize = 5;

pub(super) struct Displacements<H: ChdHasher> {
    pub(super) inner: Vec<(H::Hash, H::Hash)>,
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

pub(super) fn generate<T, H>(entries: &[T]) -> (H::Seed, Displacements<H>)
where
    T: Hash,
    H: ChdHasher,
    Standard: Distribution<H::Seed>,
    usize: AsPrimitive<H::Hash>,
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

fn try_generate<H>(hashes: &[Hashes<H>]) -> Option<Displacements<H>>
where
    H: ChdHasher,
    usize: AsPrimitive<H::Hash>,
{
    let table_len = hashes.len();
    let num_buckets = (table_len + LAMBDA - 1) / LAMBDA;

    let mut buckets: Vec<_> = (0..num_buckets).map(Bucket::new).collect();

    for (i, hash) in hashes.iter().enumerate() {
        buckets[(hash.0 % num_buckets.as_()).as_()].keys.push(i);
    }
    buckets.sort_by(|a, b| Ord::cmp(&a.keys.len(), &b.keys.len()).reverse());

    let mut displacements = vec![(H::Hash::zero(), H::Hash::zero()); num_buckets];
    let mut map = vec![false; table_len];
    let mut try_map = vec![0u64; table_len];
    let mut generation = 0;
    let mut values_to_add = Vec::with_capacity(LAMBDA);

    'buckets: for bucket in &buckets {
        for d1 in 0..table_len {
            'disps: for d2 in 0..table_len {
                values_to_add.clear();
                generation += 1;

                for &key in &bucket.keys {
                    let index = (displace::<H>(hashes[key].1, hashes[key].2, d1.as_(), d2.as_())
                        % table_len.as_())
                    .as_();

                    if map[index] || try_map[index] == generation {
                        continue 'disps;
                    }

                    try_map[index] = generation;
                    values_to_add.push(index);
                }

                displacements[bucket.index] = (d1.as_(), d2.as_());
                for &index in &values_to_add {
                    map[index] = true;
                }
                continue 'buckets;
            }
        }
        return None;
    }

    Some(Displacements {
        inner: displacements,
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

pub(super) fn displace<H>(f1: H::Hash, f2: H::Hash, d1: H::Hash, d2: H::Hash) -> H::Hash
where
    H: ChdHasher,
{
    f1.wrapping_mul(&d1).wrapping_add(&f2).wrapping_add(&d2)
}
