use crate::alloc::vec::Vec;

#[test]
fn self_test() {
    let example = vec![3, 6];
    let example_str = crate::ser::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: Vec<u64> = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: Vec<u64> = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}

#[test]
fn self_test2() {
    let example = vec![vec![2, 4], vec![56, 7]];
    let example_str = crate::ser::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: Vec<Vec<u64>> = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: Vec<Vec<u64>> = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
