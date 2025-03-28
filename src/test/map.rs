use std::collections::HashMap;

#[test]
fn self_test() {
    let example = HashMap::from([("asd", 1), ("123", 2), ("qwe rty", 3)]);
    let example_str = crate::ser::to_string(&example).unwrap();
    println!("{example_str}");

    let example_des: HashMap<&str, i32> = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: HashMap<&str, i32> = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
