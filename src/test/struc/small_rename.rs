use serde::{Deserialize, Serialize};

pub type ExType = First;
pub fn get_example() -> ExType {
    First {
        card: 25,
        asd: 2,
        text: "dsagikojdag kasdgf jfkdajklg".to_owned(),
        numbers: Second {
            conway: vec![
                SecondEnum::String("QW🔥ES".to_owned()),
                SecondEnum::U64(11),
                SecondEnum::String("asd".to_owned()),
                SecondEnum::U64(1231),
                SecondEnum::U64(100000),
                SecondEnum::U64(312211),
            ],
            fibonacci: vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34],
        },
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct First {
    #[serde(rename = "1")]
    pub card: u64,
    #[serde(rename = "2")]
    pub asd: u64,
    #[serde(rename = "3")]
    pub text: String,
    #[serde(rename = "4")]
    pub numbers: Second,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Second {
    #[serde(rename = "1")]
    pub conway: Vec<SecondEnum>,
    #[serde(rename = "2")]
    pub fibonacci: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
// #[serde(untagged)]
pub enum SecondEnum {
    #[serde(rename = "1")]
    U64(u64),
    #[serde(rename = "2")]
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
