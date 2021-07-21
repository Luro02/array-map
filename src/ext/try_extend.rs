pub trait TryExtend<A> {
    type Error;

    fn try_extend<T: IntoIterator<Item = A>>(&mut self, iter: T) -> Result<(), Self::Error>;
}
