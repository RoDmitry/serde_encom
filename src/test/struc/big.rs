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
    }]
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct A1 {
    a1: u64,
    a2: String,
    a3: Vec<String>,
    a4: Vec<u64>,
    a5: A2,
    a6: Vec<A2>,
    a8: Vec<E1>,
    a7: E1,
    a9: (u64, u64),
    a10: (A2, A2),
    a11: (E1, String),
    a12: E1,
    a13: Option<String>,
    a14: Option<E1>,
    a15: [u64; 4],
    a16: [E1; 4],
    a17: Vec<(u64, u64)>,
    a18: bool,
    a19: bool,
    a20: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct A2 {
    a1: u64,
    a2: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
// #[serde(untagged)]
pub enum E1 {
    U64(u64),
    Vecu64(Vec<u64>),
    VecStr(Vec<String>),
    Tuple((u64, u64)),
    String(String),
}

#[test]
fn self_test() {
    let example = get_example();
    let example_str = crate::ser::to_string(&example).unwrap();
    println!("{example_str}");

    let example_des: ExType = crate::des::from_slice(example_str.as_bytes()).unwrap();
    assert_eq!(example, example_des);

    let example_des: ExType = crate::des::from_str(&example_str).unwrap();
    assert_eq!(example, example_des);
}
