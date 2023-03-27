#[test]
fn self_test() {
    let example = 64;
    let example_str = crate::ser::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: u64 = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: u64 = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
