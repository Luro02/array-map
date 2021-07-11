#[macro_export]
macro_rules! arraymap_infer {
    // To support trailing commas in the macro
    ( $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr, )+ ) => {
        $crate::arraymap_infer!($( @build_hasher => $bh, )? $( $key => $value ),+)
    };
    ( @helper_construct $bh:expr ) => {
        $crate::ArrayMap::with_build_hasher($bh)
    };
    ( @helper_construct ) => {
        $crate::ArrayMap::new()
    };
    ( $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr ),*) => {
        {
            let value: Result<_, $crate::CapacityError> = (|| {
                let mut _map = $crate::arraymap_infer!( @helper_construct $( $bh )? );

                $(
                    _map.insert($key, $value)?;
                )*

                Ok(_map)
            })();

            value
        }
    };
}

#[macro_export]
macro_rules! arraymap {
    // replaces `_t` with the provided expression `e`
    (@replace $_t:tt $e:expr) => { $e };
    // counts the number of tokens and returns a const expr
    (@count $($x:expr),*) => {
        <[()]>::len(&[$( $crate::arraymap!(@replace $x ()) ),*])
    };
    // To support trailing commas in the macro
    ( $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr, )+ ) => {
        $crate::arraymap!($( @build_hasher => $bh, )? $( $key => $value ),+)
    };
    ( $( @build_hasher => $bh:expr, )? $( $key:expr => $value:expr ),*) => {
        {
            let _map: $crate::ArrayMap<_, _, { $crate::arraymap!(@count $($key),*) }, _> = $crate::arraymap_infer!(
                $( @build_hasher => $bh, )?
                $( $key => $value ),*
            ).expect("`arraymap` macro does not count correctly!");

            _map
        }
    };
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::array_map::DefaultHashBuilder;

    #[test]
    fn test_arraymap_macro_empty() {
        let map: crate::ArrayMap<(), (), 0> = arraymap!();
        assert_eq!(map, crate::ArrayMap::new());
    }

    #[test]
    fn test_arraymap_macro_infer() {
        let map = arraymap! {
            "key_00" => "value_00",
            "key_01" => "value_00",
            "key_02" => "value_00",
            "key_03" => "value_00"
        };

        assert_eq!(map, {
            let mut map = crate::ArrayMap::new();

            map.insert("key_00", "value_00").unwrap();
            map.insert("key_01", "value_00").unwrap();
            map.insert("key_02", "value_00").unwrap();
            map.insert("key_03", "value_00").unwrap();

            map
        });
    }

    #[test]
    fn test_arraymap_macro_trailing_comma() {
        arraymap! {
            "key_00" => "value_00",
            "key_01" => "value_00",
            "key_02" => "value_00",
            "key_03" => "value_00",
        };
    }

    #[test]
    fn test_arraymap_macro_build_hasher() {
        arraymap! {
            @build_hasher => DefaultHashBuilder::default(),
            "key_00" => "value_00",
            "key_01" => "value_00",
        };
    }
}
