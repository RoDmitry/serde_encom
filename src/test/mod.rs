use crate::encom_from_json;

mod bytes;
mod int;
mod option_int;
mod struc;
mod value;
mod vec_int;
mod vec_option_int;
mod vec_str;

#[test]
fn example_json() {
    let v = encom_from_json!({
        "admins": [
            {
                "adminGroupID": 1,
                "adminName": "admin",
                "balance": 10.4,
                "folder": "main",
                "number": 2
            },
            {
                "adminGroupID": 4,
                "adminName": "",
                "balance": -2.3,
                "folder": "other",
                "number": 5
            }
        ],
        "cacheFolder": null,
        "isGood": true,
        "mapping": {
            "files": "/static/*",
            "tools": "/tools/*"
        },
        "useJSON": false
    });

    println!("{v:#}");
}

#[test]
fn example_json_compact() {
    let v = encom_from_json!({"1": [{"1": 1, "2": "admin", "3": 10.4, "4": "main", "5": 2}, {"1": 4, "2": "", "3": -2.3, "4": "other", "5": 5}], "2": null, "3": true, "4": {"1": "/static/*", "2": "/tools/*"}, "5": false});
    println!("{v}");
}

/* use crate::InitDeserializer;
#[test]
fn stream() {
    let data = b"0 1 ";
    let des = crate::Deserializer::from_slice(data);
    let init_des = InitDeserializer { des: &mut des };
    let mut stream = init_des.into_iter::<Vec<i32>>();

    assert_eq!(0, stream.byte_offset());
    println!("{:?}", stream.next()); // [0]

    assert_eq!(3, stream.byte_offset());
    println!("{:?}", stream.next()); // [1]

    assert_eq!(7, stream.byte_offset());
    println!("{:?}", stream.next()); // error

    assert_eq!(8, stream.byte_offset());
    // If err.is_eof(), can join the remaining data to new data and continue.
    let remaining = &data[stream.byte_offset()..];
} */
