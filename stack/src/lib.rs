#![allow(clippy::uninit_assumed_init)]

mod stack;
mod stack_last;
mod stack_raw;

pub use crate::stack::Stack;
pub use crate::stack_last::StackLast;
pub use crate::stack_raw::StackRaw;

/* use ::std::mem::{ManuallyDrop, MaybeUninit};
macro_rules! arr_from_clone(
    ($val:expr; $len:expr)
    => {
        {
            // (this might panic... of consequence to nobody, anywhere)
            let x = $val;

            let array: [_; $len] = unsafe { MaybeUninit::uninit().assume_init() };
            let mut array = ManuallyDrop::new(array);

            // (of course, one could micro-optimize this to do $n-1 clones...)
            for p in &mut *array {
                // clone() may panic, but if this occurs then the elements
                // of array will simply be leaked without risk of dropping
                // uninitialized data.
                unsafe { ::std::ptr::write(p, x.clone()); }
            }

            ManuallyDrop::into_inner(array)
        }
    };
); */
