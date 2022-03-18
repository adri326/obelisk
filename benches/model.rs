use criterion::{black_box, criterion_group, criterion_main, Criterion};
use obelisk::*;
use obelisk::model::*;

fn bench_model(c: &mut Criterion) {
    let players = vec![
        Player::with_values(2, 1, 4, 2, 0),
        Player::with_values(4, 1, 2, 2, 0),
        Player::with_values(3, 3, 2, 2, 0),
        Player::with_values(5, 1, 1, 2, 0),
        Player::with_values(2, 7, 3, 1, 0),
        Player::with_values(4, 1, 2, 2, 0),
        Player::with_values(2, 4, 3, 2, 0),
        Player::with_values(3, 1, 2, 2, 0),
        Player::with_values(1, 5, 3, 2, 0),
        Player::with_values(2, 2, 3, 2, 0),
    ];

    let previous_actions = vec![
        Action::Skip,
        Action::Barracks,
        Action::Barracks,
        Action::Recruit,
        Action::Obelisk,
    ];

    let model = load_model("target/model.onnx").unwrap();
    const PLAYER: usize = 8;

    c.bench_function("run_model", |b| {
        b.iter(|| {
            black_box(run_model(
                &model,
                &previous_actions,
                &players,
                PLAYER,
                &players[PLAYER].possible_actions(
                    players.iter().enumerate().filter(|(n, _p)| *n != PLAYER),
                )
            ).unwrap());
        });
    });
}

criterion_group!(benches, bench_model);
criterion_main!(benches);
