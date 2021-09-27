#[macro_export]
macro_rules! array_map {
    // replaces `_t` with the provided expression `e`
    (@replace $_t:tt $e:expr) => { $e };
    // counts the number of tokens and returns a const expr
    (@count $($x:expr),*) => {
        <[()]>::len(&[$( $crate::array_map!(@replace $x ()) ),*])
    };
    // To support trailing commas in the macro
    ( @infer, $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr, )+ ) => {
        $crate::array_map!( @infer, $( @build_hasher => $bh, )? $( $key => $value ),+)
    };
    ( $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr, )+ ) => {
        $crate::array_map!( $( @build_hasher => $bh, )? $( $key => $value ),+)
    };
    ( @helper_construct $bh:expr ) => {
        $crate::ArrayMap::with_build_hasher($bh)
    };
    ( @helper_construct ) => {
        {
            #[cfg(feature = "ahash")]
            {
                $crate::ArrayMap::new()
            }
            #[cfg(not(feature = "ahash"))]
            {
                ::core::compile_error!("`ahash` feature is disabled, so a build_hasher must be specified explicitly!")
            }
        }
    };
    ( @infer, $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr ),* ) => {
        {
            let value: Result<_, $crate::CapacityError> = (|| {
                let mut _map = $crate::array_map!( @helper_construct $( $bh )? );

                $(
                    _map.try_insert($key, $value)?;
                )*

                Ok(_map)
            })();

            value
        }
    };
    ( $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr ),* ) => {
        {
            let _map: $crate::ArrayMap<_, _, { $crate::array_map!(@count $($key),*) }, _> = $crate::array_map!(
                @infer,
                $( @build_hasher => $bh, )?
                $( $key => $value ),*
            ).expect("`arraymap` macro does not count correctly!");

            _map
        }
    };
}

#[macro_export]
macro_rules! index_map {
    // replaces `_t` with the provided expression `e`
    (@replace $_t:tt $e:expr) => { $e };
    // counts the number of tokens and returns a const expr
    (@count $($x:expr),*) => {
        <[()]>::len(&[$( $crate::index_map!(@replace $x ()) ),*])
    };
    // To support trailing commas in the macro
    ( @infer, $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr, )+ ) => {
        $crate::index_map!( @infer, $( @build_hasher => $bh, )? $( $key => $value ),+)
    };
    ( $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr, )+ ) => {
        $crate::index_map!( $( @build_hasher => $bh, )? $( $key => $value ),+)
    };
    ( @helper_construct $bh:expr ) => {
        $crate::IndexMap::with_build_hasher($bh)
    };
    ( @helper_construct ) => {
        {
            #[cfg(feature = "ahash")]
            {
                $crate::IndexMap::new()
            }
            #[cfg(not(feature = "ahash"))]
            {
                ::core::compile_error!("`ahash` feature is disabled, so a build_hasher must be specified explicitly!")
            }
        }
    };
    ( @infer, $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr ),* ) => {
        {
            let value: Result<_, $crate::CapacityError> = (|| {
                let mut _map = $crate::index_map!( @helper_construct $( $bh )? );

                $(
                    _map.try_insert($key, $value)?;
                )*

                Ok(_map)
            })();

            value
        }
    };
    ( $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr ),* ) => {
        {
            let _map: $crate::IndexMap<_, _, { $crate::index_map!(@count $($key),*) }, _> = $crate::index_map!(
                @infer,
                $( @build_hasher => $bh, )?
                $( $key => $value ),*
            ).expect("`index_map` macro does not count correctly!");

            _map
        }
    };
}

#[cfg(all(test, feature = "ahash"))]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::array_map_facade::DefaultHashBuilder;
    use crate::ArrayMap;

    #[test]
    fn test_arraymap_macro_empty() {
        let map: crate::ArrayMap<(), (), 0> = array_map!();
        assert_eq!(map, crate::ArrayMap::new());
    }

    #[test]
    fn test_arraymap_macro_infer() {
        let map = array_map! {
            "key_00" => "value_00",
            "key_01" => "value_00",
            "key_02" => "value_00",
            "key_03" => "value_00"
        };

        assert_eq!(map, {
            let mut map = crate::ArrayMap::new();

            map.try_insert("key_00", "value_00").unwrap();
            map.try_insert("key_01", "value_00").unwrap();
            map.try_insert("key_02", "value_00").unwrap();
            map.try_insert("key_03", "value_00").unwrap();

            map
        });
    }

    #[test]
    fn test_arraymap_macro_trailing_comma() {
        array_map! {
            "key_00" => "value_00",
            "key_01" => "value_00",
            "key_02" => "value_00",
            "key_03" => "value_00",
        };
    }

    #[test]
    fn test_arraymap_macro_build_hasher() {
        array_map! {
            @build_hasher => DefaultHashBuilder::default(),
            "key_00" => "value_00",
            "key_01" => "value_00",
        };
    }

    #[test]
    fn test_arraymap_macro_infer_flag() {
        let _map: ArrayMap<_, _, 5> = array_map! {
            @infer,
            "hello" => "world",
            "foo" => "bar",
        }
        .unwrap();
    }
}
