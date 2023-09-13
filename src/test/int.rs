#[test]
fn u64() {
    let example = 64;
    let example_str = crate::ser::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: u64 = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: u64 = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}

#[test]
fn i64() {
    let example: i64 = -64;
    let example_str = crate::ser::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: i64 = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: i64 = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}

#[test]
fn u64_err() {
    let example_str = "64a";

    let example_des = crate::des::from_slice::<u64>(example_str.as_bytes());
    if example_des.is_ok() {
        panic!("error");
    }

    let example_des = crate::des::from_str::<u64>(example_str);
    if example_des.is_ok() {
        panic!("error");
    }
}

#[test]
fn i64_err() {
    let example_str = "-64a";

    let example_des = crate::des::from_slice::<i64>(example_str.as_bytes());
    if example_des.is_ok() {
        panic!("error");
    }

    let example_des = crate::des::from_str::<i64>(example_str);
    if example_des.is_ok() {
        panic!("error");
    }
}
