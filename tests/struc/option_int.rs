use serde::{Deserialize, Serialize};

pub type ExType = A1;
pub fn get_example() -> ExType {
    A1 {
        a1: Some(34),
        a2: None,
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct A1 {
    a1: Option<u64>,
    a2: Option<u64>,
}

#[test]
fn self_test() {
    let example = get_example();
    let example_str = serde_encom::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: ExType = serde_encom::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: ExType = serde_encom::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
