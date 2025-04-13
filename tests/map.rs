use std::collections::HashMap;

#[test]
fn self_test() {
    let example = HashMap::from([("asd", 1), ("123", 2), ("qwe rty", 3)]);
    let example_str = serde_encom::to_string(&example).unwrap();
    println!("{example_str}");

    let example_des: HashMap<&str, i32> = serde_encom::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: HashMap<&str, i32> = serde_encom::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
