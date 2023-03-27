use alloc::vec::Vec;

#[test]
fn self_test() {
    let example = vec![Some(64)];
    let example_str = crate::ser::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");
    let example_des: Vec<Option<u64>> = crate::des::from_slice(example_str.as_bytes()).unwrap();

    assert_eq!(example, example_des);
}

#[test]
fn self_test_none() {
    let example = vec![None, None, Some(33)];
    let example_str = crate::ser::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: Vec<Option<u64>> = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: Vec<Option<u64>> = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
