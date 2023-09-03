use phf_chd::Map;
use phf_shared::hash::AHasher;
use phf_shared::PhfMap;

#[test]
fn entry_multiple() {
    const ENTRIES: [(&'static str, u32); 4] =
        [("foo", 1234), ("bar", 5678), ("baz", 42424242), ("qux", 0)];

    let map = Map::<_, _, AHasher>::from_iter(ENTRIES.into_iter());

    assert_eq!(map.indices.len(), ENTRIES.len());
    assert_eq!(map.entries.len(), ENTRIES.len());
    assert_eq!(map.get_entry("foo"), Some((&"foo", &1234)));
    assert_eq!(map.get_entry("bar"), Some((&"bar", &5678)));
    assert_eq!(map.get_entry("baz"), Some((&"baz", &42424242)));
    assert_eq!(map.get_entry("qux"), Some((&"qux", &0)));
    assert_eq!(map.get_entry("other"), None);
}
