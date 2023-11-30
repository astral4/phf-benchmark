use crate::{Hasher, Seedable};
use ahash::{AHasher as AHasherBase, RandomState};
use core::hash::BuildHasher;

pub struct AHasher(AHasherBase);

impl Hasher for AHasher {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }
}

impl Seedable for AHasher {
    type Seed = (u64, u64, u64, u64);

    fn new_with_seed(seed: &Self::Seed) -> Self {
        Self(RandomState::with_seeds(seed.0, seed.1, seed.2, seed.3).build_hasher())
    }
}
