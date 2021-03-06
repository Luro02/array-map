use core::hash::{BuildHasher, Hash};
use core::mem;

use crate::raw::RawTable;
use crate::utils;

/// A draining iterator over entries of an `ArrayMap` which do not satisfy the
/// predicate `F`.
///
/// This struct is created by [`ArrayMap::drain_filter`]. See its documentation
/// for more.
///
/// [`ArrayMap::drain_filter`]: crate::ArrayMap::drain_filter
pub struct DrainFilter<'a, K, V, F, R, B>
where
    B: BuildHasher,
    F: FnMut(&K, &mut V) -> bool,
    K: Hash + Eq,
    R: RawTable<(K, V)>,
{
    f: F,
    iter: R::RawIter,
    table: &'a mut R,
    build_hasher: &'a B,
}

impl<'a, K, V, F, R, B> DrainFilter<'a, K, V, F, R, B>
where
    B: BuildHasher,
    F: FnMut(&K, &mut V) -> bool,
    K: Hash + Eq,
    R: RawTable<(K, V)>,
{
    pub(crate) fn new(f: F, table: &'a mut R, build_hasher: &'a B) -> Self {
        Self {
            f,
            iter: table.iter_idents(),
            table,
            build_hasher,
        }
    }
}

impl<'a, K, V, F, R, B> Iterator for DrainFilter<'a, K, V, F, R, B>
where
    B: BuildHasher,
    F: FnMut(&K, &mut V) -> bool,
    K: Eq + Hash,
    R: RawTable<(K, V)>,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: is it okay that the table is invalid until DrainFilter is dropped?
        for ident in &mut self.iter {
            unsafe {
                let (k, v) = self.table.get_unchecked_mut(ident.clone());
                if (self.f)(&*k, v) {
                    let result = self.table.erase(ident);
                    return Some(result);
                }
            }
        }

        None
    }
}

impl<'a, K, V, F, R, B> Drop for DrainFilter<'a, K, V, F, R, B>
where
    B: BuildHasher,
    F: FnMut(&K, &mut V) -> bool,
    K: Eq + Hash,
    R: RawTable<(K, V)>,
{
    fn drop(&mut self) {
        self.for_each(mem::drop);
        self.table.rehash(utils::key_hasher(self.build_hasher));
    }
}

#[cfg(all(test, feature = "ahash"))]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::array_map;
    use crate::ext::IteratorExt;

    #[test]
    fn test_drain_filter_drop() {
        let mut map = array_map! {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
        };

        map.drain_filter(|_, v| {
            *v = "l";
            false
        });

        assert_eq!(
            map,
            array_map! {
                0 => "l",
                1 => "l",
                2 => "l",
                3 => "l",
            }
        );
    }

    #[test]
    fn test_drain_filter() {
        let mut map = array_map! {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
        };

        let drain = map.drain_filter(|k, v| {
            if k % 2 == 0 {
                true
            } else {
                *v = "u";
                false
            }
        });

        let mut dropped: [Option<(i32, &str)>; 2] = drain.try_collect().unwrap();
        dropped.sort_unstable();

        assert_eq!(dropped, [Some((0, "a")), Some((2, "c"))]);

        assert_eq!(
            map,
            array_map! {
                @infer,
                1 => "u",
                3 => "u",
            }
            .unwrap()
        );
    }
}
