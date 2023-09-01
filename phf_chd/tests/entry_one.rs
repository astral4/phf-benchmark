use phf_chd::Map;
use phf_shared::PhfMap;

mod shared;
use shared::CustomHasher;

#[test]
fn entry_one() {
    const ENTRIES: [(u8, u8); 1] = [(123, 45)];

    let map = Map::<_, _, CustomHasher>::from_iter(ENTRIES.into_iter());

    assert_eq!(map.indices.len(), ENTRIES.len());
    assert_eq!(map.entries.len(), ENTRIES.len());
    assert_eq!(map.get_entry(&123), Some((&123, &45)));
}
