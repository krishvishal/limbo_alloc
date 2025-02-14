extern crate limbo_alloc;

use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use limbo_alloc::{Box as LimboBox, Vec as LimboVec, WrapAllocator};
use std::time::Duration;

// Struct with standard allocator
struct StdHeap {
    one: std::boxed::Box<usize>,
    two: std::boxed::Box<usize>,
    three: std::boxed::Box<usize>,
    four: std::boxed::Box<usize>,
    five: std::boxed::Box<usize>,
}

// Struct with limbo allocator
struct LimboHeap {
    one: LimboBox<usize>,
    two: LimboBox<usize>,
    three: LimboBox<usize>,
    four: LimboBox<usize>,
    five: LimboBox<usize>,
}

fn bench_alloc(c: &mut Criterion) {
    // Standard allocator benchmark
    fn std_alloc(b: &mut Bencher, iters: usize) {
        b.iter(|| {
            let mut buf = std::vec::Vec::new();
            for i in 0..iters {
                let heap = StdHeap {
                    one: std::boxed::Box::new(black_box(i)),
                    two: std::boxed::Box::new(black_box(i + 1)),
                    three: std::boxed::Box::new(black_box(i + 2)),
                    four: std::boxed::Box::new(black_box(i + 3)),
                    five: std::boxed::Box::new(black_box(i + 4)),
                };
                buf.push(heap);
            }
        })
    }

    // LimboAllocator benchmark
    fn limbo_alloc(b: &mut Bencher, iters: usize) {
        b.iter(|| {
            let wrap_alloc = WrapAllocator::new();
            let _guard = unsafe { wrap_alloc.guard() };

            let mut vec = LimboVec::new();
            for i in 0..iters {
                let heap = LimboHeap {
                    one: LimboBox::new(black_box(i)),
                    two: LimboBox::new(black_box(i + 1)),
                    three: LimboBox::new(black_box(i + 2)),
                    four: LimboBox::new(black_box(i + 3)),
                    five: LimboBox::new(black_box(i + 4)),
                };
                vec.push(heap);
            }
        })
    }

    // Run benchmarks at different sizes
    let sizes = [1000, 10000, 100000, 1000000];

    for size in sizes {
        c.bench_function(&format!("std_alloc_{}", size), |b| std_alloc(b, size));
        c.bench_function(&format!("limbo_alloc_{}", size), |b| limbo_alloc(b, size));
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(20));
    targets = bench_alloc
}

criterion_main!(benches);
