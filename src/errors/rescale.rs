use core::fmt;

pub struct RescaleError<const NEW_CAP: usize> {
    required_size: usize,
    old_capacity: usize,
}

impl<const NEW_CAP: usize> RescaleError<NEW_CAP> {
    #[must_use]
    pub(crate) const fn new(required_size: usize, old_capacity: usize) -> Self {
        Self {
            required_size,
            old_capacity,
        }
    }
}

impl<const NEW_CAP: usize> fmt::Debug for RescaleError<NEW_CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RescaleError").finish()
    }
}

impl<const NEW_CAP: usize> fmt::Display for RescaleError<NEW_CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            concat!(
                "failed to rescale the map of size `{size}` and capacity `{n}`,",
                " because the new map can hold at most `{c}` elements",
            ),
            size = self.required_size,
            n = self.old_capacity,
            c = NEW_CAP
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
            RescaleError::<4>::new(5, 6).to_string(),
            concat!(
                "failed to rescale the map of size `5` and capacity `6`,",
                " because the new map can hold at most `4` elements",
            )
        );
    }

    #[test]
    fn test_rescale_error_debug() {
        assert_eq!(
            format!("{:?}", RescaleError::<4>::new(5, 6)),
            stringify!(RescaleError)
        );
    }
}
