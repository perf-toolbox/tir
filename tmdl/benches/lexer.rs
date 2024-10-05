use criterion::{criterion_group, criterion_main, Criterion};
use tmdl::lex;

const LARGE_INSTR_TEMPLATE_INPUT: &str = include_str!("./Inputs/large_instr_template.tmdl");

fn large_instr_template(c: &mut Criterion) {
    c.bench_function("large_instr_template", |b| {
        b.iter(|| {
            let _ = lex(LARGE_INSTR_TEMPLATE_INPUT).unwrap();
        })
    });
}

criterion_group!(benches, large_instr_template);
criterion_main!(benches);
