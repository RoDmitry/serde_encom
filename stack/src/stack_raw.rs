#[derive(Debug, Clone, Copy)]
pub struct StackRaw<D, const S: usize> {
    raw_pointer: *mut [D; S],
    index: usize,
}

impl<D, const S: usize> StackRaw<D, S> {
    #[inline]
    pub fn new(raw_pointer: *mut [D; S]) -> Self {
        Self {
            raw_pointer,
            index: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, v: D) {
        unsafe {
            (*self.raw_pointer)[self.index] = v;
        }
        self.index += 1;
    }
}
