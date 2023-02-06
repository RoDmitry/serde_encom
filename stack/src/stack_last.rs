use std::mem::MaybeUninit;

#[derive(Clone, Copy, Debug)]
pub struct StackLast<D, const S: usize> {
    arr: [D; S],
    index: usize,
}

impl<D, const S: usize> StackLast<D, S> {
    #[inline]
    pub fn new(v: D) -> Self {
        let mut tmp = Self {
            arr: unsafe { MaybeUninit::uninit().assume_init() },
            index: 0,
        };
        tmp.arr[tmp.index] = v;
        tmp
    }

    #[inline]
    pub fn max_index(&self) -> usize {
        S - 1
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        S
    }

    /* #[inline]
    pub fn get_slice(&self) -> &[D] {
        &self.arr[0..=self.index]
    } */

    #[inline]
    pub fn push(&mut self, v: D) -> Option<()> {
        self.index += 1;
        if self.index > self.max_index() {
            return None;
        }
        self.arr[self.index] = v;
        Some(())
    }

    #[inline]
    pub fn last_mut(&mut self) -> &mut D {
        &mut self.arr[self.index]
    }

    #[inline]
    pub fn remove_last(&mut self) -> Option<()> {
        if self.is_empty() {
            return None;
        }
        self.index -= 1;
        Some(())
    }

    /* #[inline]
    pub fn pop(&mut self) -> Option<D> {
        let res = std::mem::replace(self.last_mut(), unsafe {
            MaybeUninit::uninit().assume_init()
        });
        self.index -= 1;
        Some(res)
    } */

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
