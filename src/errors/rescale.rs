use core::fmt;

pub struct RescaleError<const N: usize, const M: usize> {
    required_size: usize,
}

impl<const N: usize, const M: usize> RescaleError<N, M> {
    #[must_use]
    pub(crate) fn new(required_size: usize) -> Self {
        Self { required_size }
    }
}

impl<const N: usize, const M: usize> fmt::Debug for RescaleError<N, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RescaleError").finish()
    }
}

impl<const N: usize, const M: usize> fmt::Display for RescaleError<N, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            concat!(
                "failed to rescale the map of size `{size}` and capacity `{n}`,",
                " because the new map can hold at most `{m}` elements",
            ),
            size = self.required_size,
            n = N,
            m = M
        )
    }
}
