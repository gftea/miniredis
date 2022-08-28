use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jemalloc_ctl::{epoch, stats};
use miniredis::{generate_binary_tree, Solution};




#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub fn criterion_benchmark(c: &mut Criterion) {
    // many statistics are cached and only updated when the epoch is advanced.
    epoch::advance().unwrap();
    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!(
        "setup: {} bytes allocated/{} bytes resident",
        allocated, resident
    );


    c.bench_function("jemalloc heap usage", |b| {
        b.iter(|| {
            let root = generate_binary_tree(10000);

            Solution::max_depth(root);
        });
    });

    // many statistics are cached and only updated when the epoch is advanced.
    epoch::advance().unwrap();
    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!(
        "end: {} bytes allocated/{} bytes resident",
        allocated, resident
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
