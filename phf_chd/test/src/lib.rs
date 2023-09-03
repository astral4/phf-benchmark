#![allow(dead_code, unused_imports)]

use phf_chd::Map;
use phf_shared::hash::AHasher;
use phf_shared::PhfMap;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[test]
fn entry_one() {
    assert_eq!(ENTRY_ONE.get_entry(&123), Some((&123, &45)));
    assert_eq!(ENTRY_ONE.get_entry(&0), None);
}

#[test]
fn entry_multiple() {
    assert_eq!(ENTRY_MULTIPLE.get_entry("foo"), Some((&"foo", &1234)));
    assert_eq!(ENTRY_MULTIPLE.get_entry("bar"), Some((&"bar", &5678)));
    assert_eq!(ENTRY_MULTIPLE.get_entry("baz"), Some((&"baz", &42424242)));
    assert_eq!(ENTRY_MULTIPLE.get_entry("qux"), Some((&"qux", &0)));
    assert_eq!(ENTRY_MULTIPLE.get_entry("other"), None);
}

#[test]
fn entry_many() {
    assert_eq!(ENTRY_MANY.entries.len(), 10000);

    for entry in ENTRY_MANY.entries {
        assert_eq!(ENTRY_MANY.get_entry(&entry.0), Some((&entry.0, &entry.1)))
    }
}
