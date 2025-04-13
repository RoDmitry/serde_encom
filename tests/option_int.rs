#[test]
fn self_test() {
    let example = Some(64);
    let example_str = serde_encom::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");
    let example_des: Option<u64> = serde_encom::from_slice(example_str.as_bytes()).unwrap();

    assert_eq!(example, example_des);
}

#[test]
fn self_test_none() {
    let example = None;
    let example_str = serde_encom::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: Option<u64> = serde_encom::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: Option<u64> = serde_encom::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
