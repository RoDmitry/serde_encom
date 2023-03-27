use crate::alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub type ExType = A1;
pub fn get_example() -> ExType {
    A1 {
        a1: vec![0x33, 0x35],
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct A1 {
    #[serde(with = "serde_bytes")]
    a1: Vec<u8>,
}

#[test]
fn self_test() {
    let example = get_example();
    let example_str = crate::ser::to_vec(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str:X?}");

    let example_des: ExType = crate::des::from_slice(&example_str).unwrap();
    assert_eq!(example, example_des);
}
