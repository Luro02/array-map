mod map {
    use core::fmt;
    use core::hash::{BuildHasher, Hash};
    use core::marker::PhantomData;
    use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};
    use serde::ser::{Serialize, Serializer};

    use crate::ArrayMap;

    impl<K, V, B, const N: usize> Serialize for ArrayMap<K, V, N, B>
    where
        K: Serialize + Eq + Hash,
        V: Serialize,
        B: BuildHasher,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.collect_map(self)
        }
    }

    impl<'de, K, V, B, const N: usize> Deserialize<'de> for ArrayMap<K, V, N, B>
    where
        K: Deserialize<'de> + Eq + Hash,
        V: Deserialize<'de>,
        B: BuildHasher + Default,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct MapVisitor<K, V, B: BuildHasher, const N: usize> {
                marker: PhantomData<ArrayMap<K, V, N, B>>,
            }

            impl<'de, K, V, B, const N: usize> Visitor<'de> for MapVisitor<K, V, B, N>
            where
                K: Deserialize<'de> + Eq + Hash,
                V: Deserialize<'de>,
                B: BuildHasher + Default,
            {
                type Value = ArrayMap<K, V, N, B>;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a map")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    let mut result = ArrayMap::with_hasher(B::default());

                    while let Some((key, value)) = map.next_entry()? {
                        result.try_insert(key, value).map_err(A::Error::custom)?;
                    }

                    Ok(result)
                }
            }

            let visitor = MapVisitor {
                marker: PhantomData,
            };
            deserializer.deserialize_map(visitor)
        }
    }
}
