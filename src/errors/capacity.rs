use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapacityError;

impl fmt::Display for CapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not enough space")
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    use alloc::string::ToString;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_capacity_error_display() {
        assert_eq!(CapacityError.to_string(), "not enough space");
    }
}
