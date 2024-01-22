use criterion::{criterion_group, criterion_main, Criterion};
use gfa_reader::{Gfa, NCGfa};

/// Read a normal graph
fn gfa_normal(filename: &str) {
    let mut graph: Gfa<()> = Gfa::new();
    graph.parse_gfa_file(filename, true)
    //sort_vector(&mut intervals);
}

/// Read a NC graph
fn ngfa_normal(filename: &str) {
    let mut _intervals: NCGfa<()> = NCGfa::new();
    _intervals.parse_gfa_file(filename, true);
    //sort_vector(&mut intervals);
}

/// Benchmark with criterion
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat-sampling-example");
    let filename =
        "/home/svorbrugg/code/data/size5.run4.fasta.gz.f1fd09c.417fcdf.b3523fd.smooth.final.gfa";

    group.bench_function("Gfa", |b| b.iter(|| gfa_normal(filename)));
    group.bench_function("ncgfa", |b| b.iter(|| ngfa_normal(filename)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
