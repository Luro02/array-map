/// The error type for [`ArrayMap::get_each_value_mut`] and
/// [`ArrayMap::get_each_key_value_mut`].
///
/// [`ArrayMap::get_each_value_mut`]: crate::ArrayMap::get_each_value_mut
/// [`ArrayMap::get_each_key_value_mut`]: crate::ArrayMap::get_each_value_mut
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnavailableMutError {
    /// The requested entry is not present in the table.
    Absent,
    /// The requested entry is present, but a mutable reference to it was
    /// already created.
    ///
    /// This includes the index of the mutable reference in the returned array.
    Duplicate(usize),
}
