use crate::{encom_from_json, Value};

#[test]
fn test_data() {
    let data = r#"3{2:6 6{33 22}}"#;
    let v: Value = crate::from_str(data).unwrap();

    println!("{v}");
    assert_eq!(
        v,
        encom_from_json!({
            "3": {
                "2": 6,
                "6": [33, 22]
            }
        })
    );
}

#[test]
fn test_data2() {
    let data = r#"
         name:8=John Doe
         age:43
         phones{
             11=+44 1234567
             11=+44 2345678
         }
     "#;
    let v: Value = crate::from_str(data).unwrap();

    println!("{v}");
    assert_eq!(
        v,
        encom_from_json!({
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        })
    );
}

#[test]
fn test_seq_str_big() {
    let data = r#"name:8=John Doe"#;
    let v: Value = crate::from_str(data).unwrap();

    println!("{v}");
    assert_eq!(v, encom_from_json!({"name": "John Doe"}));
}

#[test]
fn test_map() {
    let data = r#"a:8"#;
    let v: Value = crate::from_str(data).unwrap();

    println!("{v}");
    assert_eq!(v, encom_from_json!({"a": 8}));
}

#[test]
fn test_seq_number() {
    let data = r#"15"#;
    let v: Value = crate::from_str(data).unwrap();

    println!("{v}");
    assert_eq!(v, encom_from_json!([15]));
}

#[test]
fn test_seq_number2() {
    let data = r#"15 66"#;
    let v: Value = crate::from_str(data).unwrap();

    println!("{v}");
    assert_eq!(v, encom_from_json!([15, 66]));
}

#[test]
fn test_seq_map2() {
    let data = r#"{a:1} {d:4}"#;
    let v: Value = crate::from_str(data).unwrap();

    println!("{v:?}");
    assert_eq!(v, encom_from_json!([{"a": 1}, {"d": 4}]));
}

#[test]
fn test_seq_str2() {
    let data = r#"2=fd 3=gfd"#;
    let v: Value = crate::from_str(data).unwrap();

    println!("{v:?}");
    assert_eq!(v, encom_from_json!(["fd", "gfd"]));
}

#[test]
fn test_seq_str() {
    let data = r#"1=a"#;
    let v: Value = crate::from_str(data).unwrap();

    println!("{v:?}");
    assert_eq!(v, encom_from_json!(["a"]));
}

#[test]
fn test_seq_bytes() {
    let data = b"1=a";
    let v: Value = crate::from_slice(data).unwrap();

    println!("{v:?}");
    assert_eq!(v, Value::Array(vec![Value::Bytes(vec![b'a'])]));
}

#[test]
fn test_null() {
    let data = b"a:n";
    let v: Value = crate::from_slice(data).unwrap();

    println!("{v:?}");
    assert_eq!(v, encom_from_json!({ "a": null }));
}

#[test]
fn test_enums() {
    // it's not possible to parse enum seq without knowing that it's an enum seq, and you don't have an enum itself
    let data = b"1:2 3:4 1:5";
    let v: Value = crate::from_slice(data).unwrap();

    println!("{v:?}");
}
