#![no_main]
use array_map::ArrayMap;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct HasHash(u64, u64);

impl core::hash::Hash for HasHash {
    fn hash<H>(&self, h: &mut H)
    where
        H: core::hash::Hasher,
    {
        h.write_u64(self.0);
    }
}

impl<'a> Arbitrary<'a> for HasHash {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, libfuzzer_sys::arbitrary::Error> {
        Ok(Self(u.arbitrary::<_>()?, u.arbitrary::<_>()?))
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        let size = core::mem::size_of::<u64>() * 2;
        (size, Some(size))
    }
}

#[derive(Default)]
struct Hasher(u64);

impl core::hash::Hasher for Hasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        unimplemented!()
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }
}

type CustomHasher = core::hash::BuildHasherDefault<Hasher>;

const INPUT_SIZE: usize = 3;
const CAPACITY: usize = 4;

fuzz_target!(|data: [(HasHash, usize); INPUT_SIZE]| {
    let mut map: ArrayMap<HasHash, usize, CAPACITY, CustomHasher> =
        ArrayMap::with_build_hasher(CustomHasher::default());
    let mut hash_map = HashMap::with_capacity_and_hasher(CAPACITY, CustomHasher::default());

    for (key, value) in core::array::IntoIter::new(data) {
        map.insert(key, value).unwrap();
        hash_map.insert(key, value);
    }

    let keys = data.map(|(k, _)| k);

    for key in core::array::IntoIter::new(keys) {
        assert_eq!(map.contains_key(&key), hash_map.contains_key(&key));
        assert_eq!(map.remove_entry(&key), hash_map.remove_entry(&key));
        for key in keys.iter() {
            assert_eq!(map.contains_key(key), hash_map.contains_key(key));
        }
    }
});
