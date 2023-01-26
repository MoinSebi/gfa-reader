use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use gfaR::Gfa;


/// Make data (as comparison)
fn data(size: usize){
    let mut _intervals = NGfa::new();
    //sort_vector(&mut intervals);
}



/// Benchmark with criterion
fn criterion_benchmark(c: &mut Criterion) {
    let f = 10000;
    println!("Benches run with vector of size 200000");
    c.bench_function("Data generation solo", |b| b.iter(|| data(f)));

}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);