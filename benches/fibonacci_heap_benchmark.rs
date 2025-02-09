use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fibonacci_heap::FibonacciHeap;

fn bench_insert(c: &mut Criterion) {
    c.bench_function("insert", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeap::new();
            for i in 0..1000 {
                heap.insert(black_box(i));
            }
        })
    });
}

fn bench_extract_min(c: &mut Criterion) {
    c.bench_function("extract_min", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeap::new();
            for i in 0..1000 {
                heap.insert(i);
            }
            for _ in 0..1000 {
                heap.extract_min();
            }
        })
    });
}

fn bench_decrease_key(c: &mut Criterion) {
    c.bench_function("decrease_key", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeap::new();
            let nodes: Vec<_> = (0..1000).map(|i| heap.insert(i)).collect();
            for node in nodes {
                let key = node.borrow().key;
                heap.decrease_key(key, black_box(key / 2));
            }
        })
    });
}

criterion_group!(benches, bench_insert, bench_extract_min, bench_decrease_key);
criterion_main!(benches);