use criterion::{criterion_group, criterion_main, Criterion};
use render_core::rhi::id_alloter::{OccupiedMarker, IdAlloterWithCountLimit};

fn id_alloter_bench(c: &mut Criterion) {
    let alloter = IdAlloterWithCountLimit::new(1000);

    c.bench_function("IdAlloterWithCountLimit.alloc", |b| {
        b.iter(|| {
            alloter.alloc();
        })
    });
}

fn occupied_marker_bench(c: &mut Criterion) {
    let mut marker = OccupiedMarker::new(1000);

    c.bench_function("OccupiedMarker.alloc", |b| {
        b.iter(|| {
            marker.alloc();
        })
    });
}

criterion_group!(benches, id_alloter_bench, occupied_marker_bench);
criterion_main!(benches);