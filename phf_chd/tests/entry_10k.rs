use phf_chd::Map;
use phf_shared::hash::AHasher;
use phf_shared::{PhfMap, FIXED_SEED};
use rand::distributions::Standard;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[test]
fn entry_many() {
    const MAP_LENGTH: usize = 10000;

    let map = Map::<u32, u32, AHasher>::from_iter(
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
