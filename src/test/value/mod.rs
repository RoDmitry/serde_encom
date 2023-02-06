use crate::{encom_from_json, Value};

#[test]
fn test_data() {
    let data = "3{2:6 6{33 22}}";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!({
        "3": {
            "2": 6,
            "6": [33, 22]
        }
    });
    assert_eq!(v, res);
    assert_eq!(v2, res);
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
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!({
        "name": "John Doe",
        "age": 43,
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    });
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_seq_str_big() {
    let data = "name:8=John Doe";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!({"name": "John Doe"});
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_map() {
    let data = "a:8";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!({"a": 8});
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_seq_number() {
    let data = "15";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!([15]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_seq_number2() {
    let data = "15 66";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!([15, 66]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_seq_map2() {
    let data = "{a:1} {d:4}";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!([{"a": 1}, {"d": 4}]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_seq_str2() {
    let data = "2=fd 3=gfd";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!(["fd", "gfd"]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_seq_str() {
    let data = "1=a";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!(["a"]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_seq_bytes() {
    let data = "1~a";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = Value::Array(vec![Value::Bytes(vec![b'a'])]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_null() {
    let data = "a:n";
    let v: Value = crate::from_str(data).unwrap();
    let v2: Value = crate::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    println!("{v2}");
    let res = encom_from_json!({ "a": null });
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_enums() {
    // it's not possible to parse enum seq without knowing that it's an enum seq, and you don't have an enum itself
    let data = "1:2 3:4 1:5";
    let v: Value = crate::from_str(data).unwrap();

    println!("{v:?}");
}
