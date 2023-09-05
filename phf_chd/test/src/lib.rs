#![allow(dead_code, unused_imports)]

use phf_chd::Map;
use phf_shared::hash::AHasher;
use phf_shared::PhfMap;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[test]
fn map_one() {
    assert_eq!(MAP_ONE.get_entry(&123), Some((&123, &45)));
    assert_eq!(MAP_ONE.get_entry(&0), None);
}

#[test]
fn map_multiple() {
    assert_eq!(MAP_MULTIPLE.get_entry("foo"), Some((&"foo", &1234)));
    assert_eq!(MAP_MULTIPLE.get_entry("bar"), Some((&"bar", &5678)));
    assert_eq!(MAP_MULTIPLE.get_entry("baz"), Some((&"baz", &42424242)));
    assert_eq!(MAP_MULTIPLE.get_entry("qux"), Some((&"qux", &0)));
    assert_eq!(MAP_MULTIPLE.get_entry("other"), None);
}

#[test]
fn map_arrays() {
    assert_eq!(MAP_ARRAYS.entries.len(), 1000);

    for entry in MAP_ARRAYS.entries {
        assert_eq!(MAP_ARRAYS.get_entry(&entry.0), Some((&entry.0, &entry.1)));
    }
}

#[test]
fn map_many() {
    assert_eq!(MAP_MANY.entries.len(), 10000);

    for entry in MAP_MANY.entries {
        assert_eq!(MAP_MANY.get_entry(&entry.0), Some((&entry.0, &entry.1)));
    }
}
