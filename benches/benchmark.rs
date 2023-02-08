use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion,
};
// use prost::Message;
use serde::{Deserialize, Serialize};
use serde_encom::Value;

type ExType = A1;
fn get_example() -> ExType {
    A1 {
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
        a13: Some("fash".to_owned()),
        a14: Some(E1::String("dgasdfgh".to_owned())),
        a15: [1, 3, 6, 8],
        a16: [E1::U64(4), E1::U64(6), E1::U64(3), E1::U64(2)],
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct A1 {
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
    #[serde(rename = "13")]
    a13: Option<String>,
    #[serde(rename = "14")]
    a14: Option<E1>,
    #[serde(rename = "15")]
    a15: [u64; 4],
    #[serde(rename = "16")]
    a16: [E1; 4],
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct A2 {
    #[serde(rename = "1")]
    a1: u64,
    #[serde(rename = "2")]
    a2: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
// #[serde(untagged)]
enum E1 {
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

/* fn bench_json_serialize(bench_group: &mut BenchmarkGroup<WallTime>, example: &ex::Start) {
    bench_group.bench_with_input(BenchmarkId::new("serialize", 1), &example, |b, &val| {
        b.iter(|| serde_json::to_string(val).unwrap())
    });
}

fn bench_json_deserialize(bench_group: &mut BenchmarkGroup<WallTime>, data: &str) {
    bench_group.bench_with_input(BenchmarkId::new("deserialize", 1), &data, |b, &val| {
        b.iter(|| serde_json::from_str::<ex::Start>(val).unwrap())
    });
} */

/* fn bench_json_serialize2(bench_group: &mut BenchmarkGroup<WallTime>, example: &ex2::ExType) {
    bench_group.bench_with_input(BenchmarkId::new("serialize", 2), &example, |b, &val| {
        b.iter(|| serde_json::to_string(val).unwrap())
    });
}

fn bench_json_deserialize2(bench_group: &mut BenchmarkGroup<WallTime>, data: &str) {
    bench_group.bench_with_input(BenchmarkId::new("deserialize", 2), &data, |b, &val| {
        b.iter(|| serde_json::from_str::<ex2::ExType>(val).unwrap())
    });
} */

/* fn bench_prost_serialize(bench_group: &mut BenchmarkGroup<WallTime>, example: &proto::First) {
    bench_group.bench_with_input(BenchmarkId::new("serialize", 1), &example, |b, &val| {
        b.iter(|| {
            let mut buf = Vec::new();
            buf.reserve(val.encoded_len());
            val.encode(&mut buf).unwrap();
            buf
        })
    });
}

fn bench_prost_deserialize(bench_group: &mut BenchmarkGroup<WallTime>, buf: &[u8]) {
    bench_group.bench_with_input(BenchmarkId::new("deserialize", 1), &buf, |b, &val| {
        b.iter(|| proto::First::decode(val))
    });
} */

/* fn bench_my_serialize(bench_group: &mut BenchmarkGroup<WallTime>, example: &ex::First) {
    bench_group.bench_with_input(BenchmarkId::new("serialize", 1), &example, |b, &val| {
        b.iter(|| serde_encom::ser::to_string(val))
    });
} */

fn bench_my_deserialize_des(bench_group: &mut BenchmarkGroup<WallTime>) {
    let ex = get_example();
    let data = serde_encom::to_string(&ex).unwrap();;
    bench_group.bench_with_input(BenchmarkId::new("deserialize", 1), data.as_bytes(), |b, val| {
        b.iter(|| serde_encom::from_slice::<A1>(val).unwrap())
    });
}

fn bench_my_deserialize_value(bench_group: &mut BenchmarkGroup<WallTime>) {
    let ex = get_example();
    let data = serde_encom::to_string(&ex).unwrap();;
    bench_group.bench_with_input(BenchmarkId::new("deserialize value", 1), data.as_bytes(), |b, val| {
        b.iter(|| serde_encom::des::from_slice::<Value>(val).unwrap())
    });
}

fn benchmark(c: &mut Criterion) {
    /* {
        let mut bench_group = c.benchmark_group("serde_json");
        let example = ex::get_example();
        // bench_json_serialize(&mut bench_group, &example);
        let example2 = ex2::get_example();
        // bench_json_serialize2(&mut bench_group, &example2);

        let data = serde_json::to_string(&example).unwrap();
        bench_json_deserialize(&mut bench_group, &data);
        let data2 = serde_json::to_string(&example2).unwrap();
        bench_json_deserialize2(&mut bench_group, &data2);

        bench_group.finish();
    } */
    /* {
        let mut bench_group = c.benchmark_group("prost");
        let example = proto::First {
            card: 25,
            asd: 2,
            text: "dsagikojdag kasdgf jfkdajklg".to_owned(),
            numbers: Some(proto::Second {
                conway: vec![
                    proto::SecondEnum {
                        second_enum: Some(proto::second_enum::SecondEnum::Str("QW🔥ES".to_owned())),
                    },
                    proto::SecondEnum {
                        second_enum: Some(proto::second_enum::SecondEnum::U64(11)),
                    },
                    proto::SecondEnum {
                        second_enum: Some(proto::second_enum::SecondEnum::Str("asd".to_owned())),
                    },
                    proto::SecondEnum {
                        second_enum: Some(proto::second_enum::SecondEnum::U64(1231)),
                    },
                    proto::SecondEnum {
                        second_enum: Some(proto::second_enum::SecondEnum::U64(100000)),
                    },
                    proto::SecondEnum {
                        second_enum: Some(proto::second_enum::SecondEnum::U64(312211)),
                    },
                ],
                fibonacci: vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34],
            }),
        };

        bench_prost_serialize(&mut bench_group, &example);

        let mut buf = Vec::new();
        buf.reserve(example.encoded_len());
        example.encode(&mut buf).unwrap();

        bench_prost_deserialize(&mut bench_group, &buf);

        bench_group.finish();
    } */
    {
        let mut bench_group = c.benchmark_group("my");
        /* let example = ex::get_example();
        let data = serde_encom::ser::to_string(&example).unwrap(); */

        // bench_my_serialize(&mut bench_group, &example);
        // bench_my_norc_serialize(&mut bench_group, &example);

        // let data = example.to_string();
        /* bench_my_deserialize(&mut bench_group, data.as_bytes());
        let example = ex2::get_example();
        let data = serde_encom::ser::to_string(&example).unwrap();
        bench_my_deserialize2(&mut bench_group, data.as_bytes()); */
        /* let example = ex4::get_example();
        let data = serde_encom::ser::to_string(&example).unwrap();
        bench_my_deserialize4(&mut bench_group, data.as_bytes()); */

        // bench_my_deserialize_des(&mut bench_group);
        bench_my_deserialize_value(&mut bench_group);

        /* bench_group.bench_with_input(BenchmarkId::new("atoi", 1), "123456789012345", |b, val| {
            b.iter(|| parse_u64(val, None).unwrap())
        });
        bench_group.bench_with_input(BenchmarkId::new("std parse", 1), "123456789012345", |b, val| {
            b.iter(|| val.parse::<u64>().unwrap())
        }); */

        bench_group.finish();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
