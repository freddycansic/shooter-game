use criterion::{criterion_group, criterion_main};

mod query_iter_benchmark;

criterion_group!(benches, query_iter_benchmark::query_iter_benchmark);
criterion_main!(benches);
