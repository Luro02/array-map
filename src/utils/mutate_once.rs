/// A wrapper around a mutable reference that enforces that the reference is only mutated once.
pub struct MutateOnce<'a, T> {
    value: &'a mut T,
    has_mutated: bool,
}

impl<'a, T> MutateOnce<'a, T> {
    #[inline]
    #[must_use]
    pub fn new(value: &'a mut T) -> Self {
        Self {
            value,
            has_mutated: false,
        }
    }

    #[inline]
    pub fn mutate<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        if self.has_mutated {
            panic!("MutateOnce::mutate: mutating a value twice");
        }

        self.has_mutated = true;
        f(self.value);
    }

    #[inline]
    #[must_use]
    pub fn into_mut(self) -> &'a mut T {
        self.value
    }
}

impl<'a, T> AsRef<T> for MutateOnce<'a, T> {
    fn as_ref(&self) -> &T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    #[should_panic]
    fn test_mutate_twice() {
        let mut value = 1234_usize;
        let mut mutable = MutateOnce::new(&mut value);
        mutable.mutate(|x| *x += 1);
        mutable.mutate(|x| *x += 1);
    }

    #[test]
    fn test_mutate() {
        let mut value = 12_usize;
        let mut mutable = MutateOnce::new(&mut value);
        mutable.mutate(|x| *x *= 4);
        assert_eq!(*mutable.as_ref(), 12_usize * 4);
    }
}