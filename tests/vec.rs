#[test]
fn test_int() {
    let example = vec![3, 6];
    let example_str = serde_encom::to_string(&example).unwrap();
    println!("{example_str}");

    let example_des: Vec<u64> = serde_encom::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: Vec<u64> = serde_encom::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}

#[test]
fn test_int2() {
    let example = vec![vec![2, 4], vec![56, 7]];
    let example_str = serde_encom::to_string(&example).unwrap();
    println!("{example_str}");

    let example_des: Vec<Vec<u64>> = serde_encom::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: Vec<Vec<u64>> = serde_encom::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}

#[test]
fn test_option_int() {
    let example = vec![Some(64)];
    let example_str = serde_encom::to_string(&example).unwrap();
    println!("{example_str}");
    let example_des: Vec<Option<u64>> = serde_encom::from_slice(example_str.as_bytes()).unwrap();

    assert_eq!(example, example_des);
}

#[test]
fn test_option_int_none() {
    let example = vec![None, None, Some(33)];
    let example_str = serde_encom::to_string(&example).unwrap();
    println!("{example_str}");

    let example_des: Vec<Option<u64>> = serde_encom::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: Vec<Option<u64>> = serde_encom::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}

#[test]
fn test_str() {
    let example = vec!["gsd", "rgt"];
    let example_str = serde_encom::to_string(&example).unwrap();
    println!("{example_str}");

    let example_des: Vec<&str> = serde_encom::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: Vec<&str> = serde_encom::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}

#[test]
fn test_chars() {
    let example = vec![['g', 's', 'd'], ['r', 'g', 't']];
    let example_str = serde_encom::to_string(&example).unwrap();
    println!("{example_str}");

    let example_des: Vec<[char; 3]> = serde_encom::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: Vec<[char; 3]> = serde_encom::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
