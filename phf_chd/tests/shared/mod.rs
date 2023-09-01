use ahash::{AHasher, RandomState};
use core::hash::{BuildHasher, Hasher};
use phf_chd::{ChdHasher, Hashes};
use phf_shared::Seedable;

pub(super) struct CustomHasher(AHasher);

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
