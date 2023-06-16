use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkGroup, SamplingMode};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use gfa_reader::{Gfa, NCGfa};


/// Read a normal graph
fn gfa_normal(filename: &str){
    let mut graph = Gfa::new();
    graph.parse_gfa_file(filename)
    //sort_vector(&mut intervals);
}

/// Read NGfa normal
fn ngfa_normal(filename: &str){
    let mut _intervals = NCGfa::new();
    _intervals.parse_gfa_file(filename)
    //sort_vector(&mut intervals);
}






/// Benchmark with criterion
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat-sampling-example");
    let filename = "/home/svorbrugg/code/data/size5.run4.fasta.gz.f1fd09c.417fcdf.b3523fd.smooth.final.gfa";

    group.bench_function("Gfa", |b| b.iter(|| gfa_normal(filename)));
    group.bench_function("Ngfa", |b| b.iter(|| ngfa_normal(filename)));
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);