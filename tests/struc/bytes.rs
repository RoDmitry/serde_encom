use crate::to_string;
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
    let example_str = serde_encom::to_vec(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{}", to_string(&example_str));

    let example_des: ExType = serde_encom::from_slice(&example_str).unwrap();
    assert_eq!(example, example_des);
}
