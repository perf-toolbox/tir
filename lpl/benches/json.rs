use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lpl::combinators::*;
use lpl::Parser;
use serde::Deserialize;

const SMALL_EN: &str = include_str!("./Inputs/small_en.json");
const MEDIUM_EN: &str = include_str!("./Inputs/medium_en.json");
const LARGE_EN: &str = include_str!("./Inputs/large_en.json");

fn json_lpl(c: &mut Criterion) {
    let mut group = c.benchmark_group("lpl");

    for (name, workload) in [
        ("small_en", SMALL_EN),
        ("medium_en", MEDIUM_EN),
        ("large_en", LARGE_EN),
    ]
    .iter()
    {
        group.throughput(Throughput::Bytes(workload.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            workload,
            |b, &workload| {
                b.iter(|| {
                    let stream: lpl::StrStream = workload.into();

                    let parser = json_parser();

                    parser.parse(stream).unwrap();
                })
            },
        );
    }
    group.finish()
}

fn json_serde(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde");

    for (name, workload) in [
        ("small_en", SMALL_EN),
        ("medium_en", MEDIUM_EN),
        ("large_en", LARGE_EN),
    ]
    .iter()
    {
        group.throughput(Throughput::Bytes(workload.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            workload,
            |b, &workload| {
                b.iter(|| {
                    let _: Vec<Client> = serde_json::from_str(workload).unwrap();
                })
            },
        );
    }
    group.finish()
}

criterion_group!(benches, json_lpl, json_serde);
criterion_main!(benches);

#[allow(dead_code)]
enum Json<'a> {
    Null,
    Bool(bool),
    Str(&'a str),
    Num(f64),
    Array(Vec<Json<'a>>),
    Object(Vec<(&'a str, Json<'a>)>),
}

impl NotTuple for Json<'_> {}

fn json_parser<'a>() -> impl lpl::Parser<'a, lpl::StrStream<'a>, Json<'a>> {
    recursive(|value: Recursive<'a, lpl::StrStream<'a>, _, _>| {
        let dict_entry = literal("\"")
            .and_then(text::take_while(|c| *c != '"'))
            .and_then(literal("\""))
            .and_then(literal(":").spaced())
            .and_then(value.clone())
            .flat()
            .map(|(_, name, _, _, obj): (_, &'a str, _, _, Json<'a>)| (name, obj))
            .label("dict entry");

        let object = literal("{")
            .spaced()
            .and_then(separated_ignore(
                dict_entry.spaced(),
                literal(",").spaced().void(),
            ))
            .and_then(literal("}").spaced())
            .flat()
            .map(|(_, pairs, _)| Json::Object(pairs))
            .label("object");

        let array = literal("[")
            .spaced()
            .and_then(separated_ignore(
                value.clone(),
                literal(",").spaced().void(),
            ))
            .and_then(literal("]").spaced())
            .flat()
            .map(|(_, arr, _)| Json::Array(arr))
            .label("array");

        let boolean = literal("true")
            .spaced()
            .map(|_| Json::Bool(true))
            .or_else(literal("false").spaced().map(|_| Json::Bool(false)))
            .label("bool");

        let null = literal("null").spaced().map(|_| Json::Null).label("null");

        let integer = lang::integer_literal(10)
            .map(|int| Json::Num(int.parse::<f64>().unwrap()))
            .label("int");
        let floating = lang::integer_literal(10)
            .and_then(literal("."))
            .and_then(lang::integer_literal(10))
            .flat()
            .map(|(a, _, b)| {
                let num = format!("{}.{}", a, b);
                Json::Num(num.parse::<f64>().unwrap())
            })
            .label("float");

        let string = text::string_literal(text::StringConfig::default())
            .map(Json::Str)
            .label("string");

        object
            .or_else(array)
            .or_else(boolean)
            .or_else(null)
            .or_else(floating)
            .or_else(integer)
            .or_else(string)
    })
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Client {
    client_id: u32,
    name: String,
    email: String,
    age: u8,
    address: Address,
}
