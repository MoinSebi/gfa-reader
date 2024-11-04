use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::SliceRandom;
use gfa_reader::{Gfa, SampleType};

/// Read a normal graph
fn gfa_normal(filename: &str) {
    let _gfa: Gfa<u64, (), ()> = Gfa::parse_gfa_file(filename);
}

fn gfa_id(filename: &str) {
    let _gfa: Gfa<u64, (), ()> = Gfa::parse_gfa_file(filename);
    let mut ff = _gfa.segments.clone();
    ff.shuffle(&mut rand::thread_rng());
    for x in ff.iter() {
        let _ = _gfa.get_node_nondigit(&x.id);
    }
}
fn gfa_id2(filename: &str) {
    let _gfa: Gfa<u64, (), ()> = Gfa::parse_gfa_file(filename);
    let mut ff = _gfa.segments.clone();
    ff.shuffle(&mut rand::thread_rng());
    for x in ff.iter() {
        let _ = _gfa.get_node_digit(&x.id.get_usize());
    }
}

/// Benchmark with criterion
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat-sampling-example");
    let filename =
        "data/size5.gfa";

    group.bench_function("Gfa", |b| b.iter(|| gfa_normal(filename)));
    group.bench_function("node1", |b| b.iter(|| gfa_id(filename)));
    group.bench_function("node2", |b| b.iter(|| gfa_id2(filename)));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
