/// Try to extend a collection with the contents of an iterator.
pub trait TryExtend<A> {
    /// The error returned if it failed to extend the collection.
    type Error;

    /// Tries to extend the collection with the values from the provided
    /// iterator.
    ///
    /// # Errors
    ///
    /// If the collection runs out of capacity or for example one tries to
    /// insert elements that should not be inserted.
    /// This depends mostly on the collection itself.
    fn try_extend<T: IntoIterator<Item = A>>(&mut self, iter: T) -> Result<(), Self::Error>;
}
