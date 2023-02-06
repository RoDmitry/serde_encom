use std::mem::MaybeUninit;

#[derive(Clone, Copy, Debug)]
pub struct Stack<D, const S: usize> {
    arr: [D; S],
    index: usize,
}

impl<D, const S: usize> Default for Stack<D, S> {
    fn default() -> Self {
        Self::new()
    }
}

/* , D: Index<usize, Output = O> + IndexMut<usize> + Sized */
impl<D, const S: usize> Stack<D, S> {
    #[inline]
    pub fn new() -> Self {
        Self {
            arr: unsafe { MaybeUninit::uninit().assume_init() },
            index: 0,
        }
    }

    #[inline]
    pub fn max_index(&self) -> usize {
        S - 1
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        S
    }

    #[inline]
    pub fn get_slice(&self) -> &[D] {
        &self.arr[0..self.index]
    }

    #[inline]
    pub fn push(&mut self, v: D) -> Option<()> {
        self.arr[self.index] = v;
        self.index += 1;
        if self.index > self.max_index() {
            None
        } else {
            Some(())
        }
    }

    #[inline]
    pub fn push_unchecked(&mut self, v: D) {
        self.arr[self.index] = v;
        self.index += 1;
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut D> {
        if self.is_empty() {
            return None;
        }
        Some(&mut self.arr[self.index - 1])
    }

    #[inline]
    pub fn last_mut_unchecked(&mut self) -> &mut D {
        &mut self.arr[self.index - 1]
    }

    #[inline]
    pub fn remove_last(&mut self) -> Option<()> {
        if self.is_empty() {
            return None;
        }
        self.index -= 1;
        Some(())
    }

    #[inline]
    pub fn remove_last_not_empty(&mut self) -> Option<()> {
        self.index -= 1;
        if self.is_empty() {
            return None;
        }
        Some(())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<D> {
        let res = std::mem::replace(self.last_mut()?, unsafe {
            MaybeUninit::uninit().assume_init()
        });
        self.index -= 1;
        Some(res)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.index = 0;
    }

    #[inline]
    pub fn not_empty(&mut self) -> bool {
        self.index > 0
    }

    #[inline]
    pub fn is_empty(&mut self) -> bool {
        self.index == 0
    }
}
