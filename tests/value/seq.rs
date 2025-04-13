use serde_encom::{encom_from_json, Number, Value};

#[test]
fn test_number() {
    let data = "15";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!([15]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_number2() {
    let data = "15 66";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!([15, 66]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_number_neg() {
    let data = "-15";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!([-15]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_number_neg_err() {
    let data = "-15a";
    let v = serde_encom::from_str::<Value>(data);
    if v.is_ok() {
        panic!("error");
    }

    let v = serde_encom::from_slice::<Value>(data.as_bytes());
    if v.is_ok() {
        panic!("error");
    }
}

#[test]
fn test_number_float() {
    let data = "1.5";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!([1.5]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_number_neg_float() {
    let data = "-1.5";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!([-1.5]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_map2() {
    let data = "{a:1} {d:4}";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!([{"a": 1}, {"d": 4}]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_str() {
    let data = "1=a";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!(["a"]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_str2() {
    let data = "2=fd 3=gfd";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!(["fd", "gfd"]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_bytes() {
    let data = "1~a";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = Value::Array(vec![Value::Bytes(vec![b'a'])]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_bytes2() {
    let data = "1~a 1~b";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = Value::Array(vec![Value::Bytes(vec![b'a']), Value::Bytes(vec![b'b'])]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_zero_str() {
    let data = "0=";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!([""]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_zero_str2() {
    let data = "0= 0=";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!(["", ""]);
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_enum() {
    // it's not possible to parse enum seq without knowing that it's an enum seq, and you don't have an enum itself
    let data = "1:2 3:4 1:5";
    let v: Value = serde_encom::from_str(data).unwrap();
    println!("{v:?}");
}
