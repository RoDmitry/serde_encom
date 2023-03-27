use serde::{Deserialize, Serialize};

pub type ExType<'a> = B1<'a>;
pub fn get_example() -> ExType<'static> {
    B1 { b1: "asdfg" }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct B1<'a> {
    #[serde(rename = "1")]
    b1: &'a str,
}

#[test]
fn self_test() {
    let example = get_example();
    let example_str = crate::ser::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: ExType = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: ExType = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
