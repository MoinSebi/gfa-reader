use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkGroup, SamplingMode};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use gfa_reader::{Gfa, NGfa, NGfa2, read_file_in_parallel};


/// Read a normal graph
fn gfa_normal(filename: &str){
    let mut graph = Gfa::new();
    graph.read_file(filename)
    //sort_vector(&mut intervals);
}

/// Read NGfa normal
fn ngfa_normal(filename: &str){
    let mut _intervals = NGfa::new();
    _intervals.read_file(filename)
    //sort_vector(&mut intervals);
}

/// Read NGfa normal
fn ngfa_normal_noedges(filename: &str){
    let mut graph = NGfa::new();
    graph.read_file_m1(filename)
    //sort_vector(&mut intervals);
}

/// Read NGfa normal
fn ngfa_normal_match(filename: &str){
    let mut graph = NGfa::new();
    graph.read_file_m2(filename)
    //sort_vector(&mut intervals);
}

/// Read NGfa normal
fn ngfa_normal_noedges_match(filename: &str){
    let mut graph = NGfa::new();
    graph.read_file_m3(filename)
    //sort_vector(&mut intervals);
}


/// Read NGfa normal
fn ngfa_normal_noedges_string(filename: &str){
    let mut graph = NGfa::new();
    graph.read_file_string(filename)
    //sort_vector(&mut intervals);
}

/// Read NGfa normal
fn ngfa_normal_noedges_other(filename: &str){
    let mut graph = NGfa2::new();
    graph.read_file_string(filename)
    //sort_vector(&mut intervals);
}







/// Benchmark with criterion
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat-sampling-example");
    let filename = "/home/svorbrugg/code/data/size5.run4.fasta.gz.f1fd09c.417fcdf.b3523fd.smooth.final.gfa";

    group.bench_function("Gfa", |b| b.iter(|| gfa_normal(filename)));
    group.bench_function("Ngfa", |b| b.iter(|| ngfa_normal(filename)));
    group.bench_function("Ngfa2", |b| b.iter(|| ngfa_normal_noedges(filename)));

    group.bench_function("Ngfa no edges", |b| b.iter(|| ngfa_normal_noedges(filename)));
    group.bench_function("Ngfa no edges noll", |b| b.iter(|| ngfa_normal_noedges_match(filename)));
    group.bench_function("Ngfa string", |b| b.iter(|| ngfa_normal_noedges_string(filename)));


    group.bench_function("Ngfa string nodnoassnda", |b| b.iter(|| ngfa_normal_noedges_other(filename)));

}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);