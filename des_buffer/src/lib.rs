use stack::Stack;

#[derive(Clone, Copy, Debug)]
pub struct Buffer<'a, D, const S: usize> {
    pub slice: &'a [D],
    index: usize,
    pub stack: Stack<D, S>,
    pub stack_save: Stack<D, S>,
}

impl<'a, D: Copy, const S: usize> Buffer<'a, D, S> {
    #[inline]
    pub fn new(slice: &'a [D]) -> Self {
        let stack = Stack::new();
        Self {
            slice,
            index: 0,
            stack,
            stack_save: stack,
        }
    }

    #[inline]
    pub fn get_current(&mut self) -> D {
        self.slice[self.index]
    }

    #[inline]
    pub fn next(&mut self) -> Option<D> {
        if self.index < self.slice.len() {
            let ch = self.slice[self.index];
            self.index += 1;
            Some(ch)
        } else {
            None
        }
    }

    #[inline]
    pub fn peak_by(&mut self, incr: usize) -> Option<D> {
        self.index += incr;
        if self.index < self.slice.len() {
            Some(self.slice[self.index])
        } else {
            None
        }
    }

    #[inline]
    pub fn index_increment(&mut self, incr: usize) {
        self.index += incr;
    }

    #[inline]
    pub fn index_incr(&mut self) {
        self.index += 1;
    }

    #[inline]
    pub fn get_index(&mut self) -> usize {
        self.index
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    #[inline]
    pub fn unknown_char(&mut self, ch: D) {
        self.stack.push(ch).unwrap();
    }

    /* #[inline]
    fn max_index(&self) -> usize {
        S - 1
    }

    #[inline]
    fn capacity(&self) -> usize {
        S
    } */
}
