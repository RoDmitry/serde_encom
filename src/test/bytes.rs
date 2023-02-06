#[test]
fn look_at_bytes() {
    let encom_data = b"3~\xe5\x00\xe5";
    let bytes: &[u8] = crate::from_slice(encom_data).unwrap();

    println!("{bytes:X?}");
    assert_eq!(b'\xe5', bytes[0]);
    assert_eq!(b'\0', bytes[1]);
    assert_eq!(b'\xe5', bytes[2]);
}
