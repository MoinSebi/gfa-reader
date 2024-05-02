use criterion::{criterion_group, criterion_main, Criterion};
use gfa_reader::Gfa;

/// Read a normal graph
fn gfa_normal(filename: &str) {
    let _gfa: Gfa<u64, (), ()> = Gfa::parse_gfa_file(filename);
}

/// Benchmark with criterion
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat-sampling-example");
    let filename =
        "/home/svorbrugg/code/data/size5.run4.fasta.gz.f1fd09c.417fcdf.b3523fd.smooth.final.gfa";

    group.bench_function("Gfa", |b| b.iter(|| gfa_normal(filename)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
