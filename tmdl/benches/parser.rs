use criterion::{criterion_group, criterion_main, Criterion};
use tmdl::{lex, parse};

const LARGE_INSTR_TEMPLATE_INPUT: &'static str = include_str!("./Inputs/large_instr_template.tmdl");

fn large_instr_template(c: &mut Criterion) {
    c.bench_function("large_instr_template", |b| b.iter(|| {
        let tokens = lex(LARGE_INSTR_TEMPLATE_INPUT).unwrap();
        let _ = parse(&tokens);
    }));
}

criterion_group!(benches, large_instr_template);
criterion_main!(benches);
