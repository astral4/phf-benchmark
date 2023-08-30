use crate::{PhfMap, Seedable};
use core::borrow::Borrow;
use core::hash::Hash;
use num_traits::{AsPrimitive, Unsigned};
use rand::distributions::{Distribution, Standard};

mod generate;

trait ChdHasher: Seedable {
    type Hash: Unsigned + AsPrimitive<usize>;

    fn finish_triple(&self) -> Hashes<Self>;
}

type Hashes<H> = (
    <H as ChdHasher>::Hash,
    <H as ChdHasher>::Hash,
    <H as ChdHasher>::Hash,
);

struct Map<K: 'static, V: 'static, H: Seedable> {
    seed: H::Seed,
    disps: &'static [(usize, usize)],
    entries: &'static [(K, V)],
}

impl<K, V, H> Map<K, V, H>
where
    K: Hash,
    H: ChdHasher,
    Standard: Distribution<H::Seed>,
{
    fn from_iter<I: Iterator<Item = (K, V)>>(entries: I) -> Self {
        let entries: Vec<_> = entries.collect();
        let keys: Vec<_> = entries.iter().map(|entry| &entry.0).collect();
        let (seed, state) = generate::generate::<_, H>(&keys);

        Self {
            seed,
            disps: state.displacements.leak(),
            entries: entries.leak(),
        }
    }
}

impl<K, V, H: ChdHasher> PhfMap for Map<K, V, H> {
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
        let (d1, d2) = self.disps[hashes.0.as_() % self.disps.len()];
        let index = generate::displace(hashes.1.as_(), hashes.2.as_(), d1, d2) % self.entries.len();
        // maybe d1.into(), d2.into() instead?
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
    use crate::{PhfMap, Seedable};
    use ahash::{AHasher, RandomState};
    use core::hash::{BuildHasher, Hasher};

    struct MyHasher(AHasher);

    impl Hasher for MyHasher {
        fn finish(&self) -> u64 {
            self.0.finish()
        }

        fn write(&mut self, bytes: &[u8]) {
            self.0.write(bytes)
        }
    }

    impl Seedable for MyHasher {
        type Seed = u64;

        fn new_with_seed(seed: Self::Seed) -> Self {
            Self(RandomState::with_seeds(seed, seed, seed, seed).build_hasher())
        }
    }

    impl ChdHasher for MyHasher {
        type Hash = u16;

        fn finish_triple(&self) -> Hashes<Self> {
            let output = self.0.finish();
            ((output >> 32) as u16, (output >> 16) as u16, output as u16)
        }
    }

    #[test]
    fn entry_one() {
        const ENTRIES: [(u8, u8); 1] = [(123, 45)];

        let map = Map::<_, _, MyHasher>::from_iter(ENTRIES.into_iter());

        assert_eq!(map.entries.len(), ENTRIES.len());
        assert_eq!(map.get_entry(&123), Some((&123, &45)));
    }

    #[test]
    fn foo() {
        const ENTRIES: [(u8, &'static str); 4] =
            [(1, "foo"), (208, "bar"), (39, "baz"), (74, "qux")];

        let map = Map::<_, _, MyHasher>::from_iter(ENTRIES.into_iter());

        assert_eq!(map.entries.len(), ENTRIES.len());
        assert_eq!(map.get_entry(&1), Some((&1, &"foo")));
        assert_eq!(map.get_entry(&208), Some((&208, &"bar")));
        assert_eq!(map.get_entry(&39), Some((&39, &"baz")));
        assert_eq!(map.get_entry(&74), Some((&74, &"qux")));
        assert_eq!(map.get_entry(&0), None);
    }

    #[test]
    fn entry_multiple() {
        const ENTRIES: [(&'static str, u32); 4] =
            [("foo", 1234), ("bar", 5678), ("baz", 42424242), ("qux", 0)];

        let map = Map::<_, _, MyHasher>::from_iter(ENTRIES.into_iter());

        assert_eq!(map.entries.len(), ENTRIES.len());
        assert_eq!(map.get_entry("foo"), Some((&"foo", &1234)));
        assert_eq!(map.get_entry("bar"), Some((&"bar", &5678)));
        assert_eq!(map.get_entry("baz"), Some((&"bar", &42424242)));
        assert_eq!(map.get_entry("qux"), Some((&"qux", &0)));
        assert_eq!(map.get_entry("other"), None);
    }
}
