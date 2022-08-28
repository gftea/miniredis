use criterion::{black_box, criterion_group, criterion_main, Criterion};
use miniredis::{generate_binary_tree, Solution};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("max depth of binarytree", |b| {
        b.iter(|| {
            let root = generate_binary_tree(black_box(10000));

            Solution::max_depth(root)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
