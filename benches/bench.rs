use criterion::{criterion_group, criterion_main, Criterion};
use gfa_reader::{Gfa, Segment};
use rand::prelude::SliceRandom;

/// Read a normal graph
fn gfa_normal(filename: &str) {
    let _gfa: Gfa<u64, (), ()> = Gfa::parse_gfa_file(filename);
}

/// Check get_node function wit nondigit
fn id_non_digit(ff: &Vec<Segment<u64, ()>>, gfa: &Gfa<u64, (), ()>) {
    for x in ff.iter() {
        let _ = gfa.get_segment_nondigit(&x.id);
        let _b = gfa.get_sequence_by_id_nondigit(&x.id);
    }
}

/// Check get_node function wit digit
fn id_digit(ff: &Vec<Segment<u64, ()>>, gfa: &Gfa<u64, (), ()>) {
    for x in ff.iter() {
        let _ = gfa.get_segment_digit(&x.id);
        let _b = gfa.get_sequence_by_digit(&x.id);

    }
}

/// Benchmark with criterion
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat-sampling-example");
    let filename = "data/size5.gfa";
    let graph: Gfa<u64, (), ()> = Gfa::parse_gfa_file(filename);
    let mut ff = graph.segments.clone();
    ff.shuffle(&mut rand::thread_rng());
    group.bench_function("Reader_bench", |b| b.iter(|| gfa_normal(filename)));
    group.bench_function("segment_digit", |b| b.iter(|| id_digit(&ff, &graph)));
    group.bench_function("segment_nondigit ", |b| b.iter(|| id_non_digit(&ff, &graph)));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
