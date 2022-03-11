use criterion::{black_box, criterion_group, criterion_main, Criterion};
use obelisk::*;

fn bench_update(c: &mut Criterion) {
    let state = vec![Player::new(); 12];

    let decisions_0 = [
        Action::Wall,
        Action::Barracks,
        Action::Barracks,
        Action::Obelisk,
        Action::Barracks,
        Action::Attack(1),
        Action::Barracks,
        Action::Wall,
        Action::Wall,
        Action::Barracks,
        Action::Attack(1),
        Action::Skip,
    ];

    c.bench_function("update", |b| {
        b.iter(|| {
            black_box(update(state.clone(), &decisions_0));
        });
    });
}

criterion_group!(benches, bench_update);
criterion_main!(benches);
