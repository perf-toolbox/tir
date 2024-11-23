use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tmdl::lex;

const LARGE_INSTR_TEMPLATE_INPUT: &str = include_str!("./Inputs/large_instr_template.tmdl");

fn large_instr_template(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_instr_template");
    group.throughput(Throughput::Bytes(LARGE_INSTR_TEMPLATE_INPUT.len() as u64));
    group.bench_function("lex", |b| {
        b.iter(|| {
            let _ = lex(LARGE_INSTR_TEMPLATE_INPUT).unwrap();
        })
    });
    group.finish()
}

criterion_group!(benches, large_instr_template);
criterion_main!(benches);
