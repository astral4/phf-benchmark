use crate::{PhfMap, Seedable};
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::hash::Hash;
use num_traits::bounds::UpperBounded;
use num_traits::{AsPrimitive, Unsigned, WrappingAdd, WrappingMul, Zero};
use rand::distributions::{Distribution, Standard};
use usize_cast::IntoUsize;

mod generate;

trait ChdHasher: Seedable {
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

type Hashes<H> = (
    <H as ChdHasher>::Hash,
    <H as ChdHasher>::Hash,
    <H as ChdHasher>::Hash,
);

struct Map<K: 'static, V: 'static, H: ChdHasher> {
    seed: H::Seed,
    disps: &'static [(H::Hash, H::Hash)],
    indices: &'static [usize],
    entries: &'static [(K, V)],
}

impl<K, V, H> Map<K, V, H>
where
    K: Hash,
    H: ChdHasher,
    Standard: Distribution<H::Seed>,
    usize: AsPrimitive<H::Hash>,
{
    fn from_iter<I: Iterator<Item = (K, V)>>(entries: I) -> Self {
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
            generate::displace::<H>(hashes.1, hashes.2, d1, d2).into_usize() % self.indices.len();
        let index = self.indices[indices_index];
        let entry = &self.entries[index];

        if entry.0.borrow() == key {
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ChdHasher, Hashes, Map};
    use crate::{PhfMap, Seedable, FIXED_SEED};
    use ahash::{AHasher, RandomState};
    use core::hash::{BuildHasher, Hasher};
    use rand::distributions::Standard;
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    struct CustomHasher(AHasher);

    impl Hasher for CustomHasher {
        fn finish(&self) -> u64 {
            self.0.finish()
        }

        fn write(&mut self, bytes: &[u8]) {
            self.0.write(bytes)
        }
    }

    impl Seedable for CustomHasher {
        type Seed = (u64, u64, u64, u64);

        fn new_with_seed(seed: Self::Seed) -> Self {
            Self(RandomState::with_seeds(seed.0, seed.1, seed.2, seed.3).build_hasher())
        }
    }

    impl ChdHasher for CustomHasher {
        type Hash = u16;

        fn finish_triple(&self) -> Hashes<Self> {
            let output = self.0.finish();
            ((output >> 32) as u16, (output >> 16) as u16, output as u16)
        }
    }

    #[test]
    fn entry_one() {
        const ENTRIES: [(u8, u8); 1] = [(123, 45)];

        let map = Map::<_, _, CustomHasher>::from_iter(ENTRIES.into_iter());

        assert_eq!(map.indices.len(), ENTRIES.len());
        assert_eq!(map.entries.len(), ENTRIES.len());
        assert_eq!(map.get_entry(&123), Some((&123, &45)));
    }

    #[test]
    fn entry_multiple() {
        const ENTRIES: [(&'static str, u32); 4] =
            [("foo", 1234), ("bar", 5678), ("baz", 42424242), ("qux", 0)];

        let map = Map::<_, _, CustomHasher>::from_iter(ENTRIES.into_iter());

        assert_eq!(map.indices.len(), ENTRIES.len());
        assert_eq!(map.entries.len(), ENTRIES.len());
        assert_eq!(map.get_entry("foo"), Some((&"foo", &1234)));
        assert_eq!(map.get_entry("bar"), Some((&"bar", &5678)));
        assert_eq!(map.get_entry("baz"), Some((&"baz", &42424242)));
        assert_eq!(map.get_entry("qux"), Some((&"qux", &0)));
        assert_eq!(map.get_entry("other"), None);
    }

    #[test]
    fn entry_many() {
        const MAP_LENGTH: usize = 10000;

        let map = Map::<u32, u32, CustomHasher>::from_iter(
            SmallRng::seed_from_u64(FIXED_SEED)
                .sample_iter(Standard)
                .take(MAP_LENGTH),
        );

        assert_eq!(map.indices.len(), MAP_LENGTH);
        assert_eq!(map.entries.len(), MAP_LENGTH);
        for entry in map.entries {
            assert_eq!(map.get_entry(&entry.0), Some((&entry.0, &entry.1)))
        }
    }
}
