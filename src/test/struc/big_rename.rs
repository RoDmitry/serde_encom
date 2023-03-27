use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub type ExType = Vec<A1>;
pub fn get_example() -> ExType {
    vec![A1 {
        a1: 3,
        a2: "asd".to_owned(),
        a3: vec!["gds".to_owned(), "tmuj".to_owned()],
        a4: vec![245, 45],
        a5: A2 {
            a1: 21,
            a2: "df".to_owned(),
        },
        a6: vec![
            A2 {
                a1: 65,
                a2: "ku".to_owned(),
            },
            A2 {
                a1: 87,
                a2: "h🔥ty".to_owned(),
            },
        ],
        a8: vec![
            E1::U64(7),
            E1::Vecu64(vec![2, 5, 7]),
            E1::U64(9),
            E1::VecStr(vec!["aasd".to_owned(), "gg".to_owned(), "lk".to_owned()]),
        ],
        a7: E1::U64(3),
        a9: (89, 90),
        a10: (
            A2 {
                a1: 65,
                a2: "ku".to_owned(),
            },
            A2 {
                a1: 87,
                a2: "hty".to_owned(),
            },
        ),
        a11: (E1::U64(23), "afds".to_owned()),
        a12: E1::Tuple((32, 543)),
        a13: Some("fash".to_owned()),
        a14: Some(E1::String("dgasdfgh".to_owned())),
        a15: [1, 3, 6, 8],
        a16: [E1::U64(4), E1::U64(6), E1::U64(3), E1::U64(2)],
        a17: vec![(1, 2), (3, 4)],
        a18: true,
        a19: false,
        a20: None,
        a21: -1,
        a22: 1.5,
        a23: -1.5,
    }]
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct A1 {
    #[serde(rename = "1")]
    a1: u64,
    #[serde(rename = "2")]
    a2: String,
    #[serde(rename = "3")]
    a3: Vec<String>,
    #[serde(rename = "4")]
    a4: Vec<u64>,
    #[serde(rename = "5")]
    a5: A2,
    #[serde(rename = "6")]
    a6: Vec<A2>,
    #[serde(rename = "8")]
    a8: Vec<E1>,
    #[serde(rename = "7")]
    a7: E1,
    #[serde(rename = "9")]
    a9: (u64, u64),
    #[serde(rename = "q")]
    a10: (A2, A2),
    #[serde(rename = "w")]
    a11: (E1, String),
    #[serde(rename = "e")]
    a12: E1,
    #[serde(rename = "r")]
    a13: Option<String>,
    #[serde(rename = "t")]
    a14: Option<E1>,
    #[serde(rename = "y")]
    a15: [u64; 4],
    #[serde(rename = "u")]
    a16: [E1; 4],
    #[serde(rename = "i")]
    a17: Vec<(u64, u64)>,
    #[serde(rename = "o")]
    a18: bool,
    #[serde(rename = "p")]
    a19: bool,
    #[serde(rename = "a")]
    a20: Option<u64>,
    #[serde(rename = "s")]
    a21: i64,
    #[serde(rename = "d")]
    a22: f64,
    #[serde(rename = "f")]
    a23: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct A2 {
    #[serde(rename = "1")]
    a1: u64,
    #[serde(rename = "2")]
    a2: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
// #[serde(untagged)]
pub enum E1 {
    #[serde(rename = "1")]
    U64(u64),
    #[serde(rename = "2")]
    Vecu64(Vec<u64>),
    #[serde(rename = "3")]
    VecStr(Vec<String>),
    #[serde(rename = "4")]
    Tuple((u64, u64)),
    #[serde(rename = "5")]
    String(String),
}

#[test]
fn self_test() {
    let example = get_example();
    let example_str = crate::ser::to_string(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: ExType = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: ExType = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}

#[test]
fn self_test_no_none() {
    let example = get_example();
    let example_str = "{1:3 2:3=asd 3{3=gds 4=tmuj} 4{245 45} 5{1:21 2:2=df} 6{{1:65 2:2=ku} {1:87 2:7=h🔥ty}} 8{1:7 2{2 5 7} 1:9 3{4=aasd 2=gg 2=lk}} 7{1:3} 9{89 90} q{{1:65 2:2=ku} {1:87 2:3=hty}} w{1:23 4=afds} e{4{32 543}} r:4=fash t{5:8=dgasdfgh} y{1 3 6 8} u{1:4 1:6 1:3 1:2} i{{1 2} {3 4}} o:t p:f s:-1 d:1.5 f:-1.5}";
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: ExType = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: ExType = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}

#[test]
fn self_test_preaty() {
    let example = get_example();
    let example_str = crate::ser::to_string_pretty(&example).unwrap();
    #[cfg(feature = "std")]
    println!("{example_str}");

    let example_des: ExType = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: ExType = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
