use serde_encom::{encom_from_json, Value};

#[test]
fn test_data() {
    let data = "3{2:6 6[33 22]}";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
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
         phones[
             11=+44 1234567
             11=+44 2345678
         ]
     "#;
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
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
fn test_str() {
    let data = "name:8=John Doe";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({"name": "John Doe"});
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_slash() {
    let data = "name/surname:8=John Doe";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({"name/surname": "John Doe"});
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_zero_str() {
    let data = "str:0=";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({"str": ""});
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_zero_str2() {
    let data = "str:0= str2:0=";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({"str": "", "str2": ""});
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_map() {
    let data = "a:8";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({"a": 8});
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_null() {
    let data = "a:n";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({ "a": null });
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_null2() {
    let data = "a:n b:n";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({ "a": null, "b": null });
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_number_neg() {
    let data = "a:-15";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({ "a": -15 });
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_number_float() {
    let data = "a:1.5";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({ "a": 1.5 });
    assert_eq!(v, res);
    assert_eq!(v2, res);
}

#[test]
fn test_number_neg_float() {
    let data = "a:-1.5";
    let v: Value = serde_encom::from_str(data).unwrap();
    let v2: Value = serde_encom::from_slice(data.as_bytes()).unwrap();

    println!("{v}");
    let res = encom_from_json!({ "a": -1.5 });
    assert_eq!(v, res);
    assert_eq!(v2, res);
}
