use crate::invariant;

#[derive(Copy, Clone)]
pub struct TableIndex<const N: usize>(usize);

impl<const N: usize> TableIndex<N> {
    #[must_use]
    pub(crate) unsafe fn new(index: usize) -> Self {
        invariant!(index < N);
        Self(index)
    }

    #[must_use]
    pub(crate) fn index(&self) -> usize {
        self.0
    }
}
