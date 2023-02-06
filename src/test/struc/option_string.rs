use serde::{Deserialize, Serialize};

pub type ExType = A1;
pub fn get_example() -> ExType {
    A1 {
        a1: Some("asd".to_owned()),
        a2: None,
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct A1 {
    a1: Option<String>,
    a2: Option<String>,
}

#[test]
fn self_test() {
    let example = get_example();
    let example_str = crate::ser::to_string(&example).unwrap();
    println!("{example_str}");

    let example_des: ExType = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: ExType = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
