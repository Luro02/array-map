use core::fmt;

pub struct RescaleError<const N: usize, const M: usize> {
    required_size: usize,
}

impl<const N: usize, const M: usize> RescaleError<N, M> {
    #[must_use]
    pub(crate) const fn new(required_size: usize) -> Self {
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

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    use alloc::format;
    use alloc::string::ToString;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_rescale_error_display() {
        assert_eq!(
            RescaleError::<6, 4>::new(5).to_string(),
            concat!(
                "failed to rescale the map of size `5` and capacity `6`,",
                " because the new map can hold at most `4` elements",
            )
        );
    }

    #[test]
    fn test_rescale_error_debug() {
        assert_eq!(
            format!("{:?}", RescaleError::<6, 4>::new(5)),
            stringify!(RescaleError)
        );
    }
}
