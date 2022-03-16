use obelisk::*;
// use obelisk::monte_carlo::*;
use obelisk::model::*;

fn main() {
    let model = load_model("target/model.onnx").unwrap();

    let players = vec![
        Player::with_values(2, 1, 3, 2, 0),
        Player::with_values(3, 1, 2, 2, 0),
        Player::with_values(3, 3, 2, 1, 0),
        Player::with_values(4, 1, 1, 2, 0),
        Player::with_values(2, 4, 3, 1, 0),
        Player::with_values(3, 1, 2, 2, 0),
        Player::with_values(2, 5, 2, 1, 0),
        Player::with_values(2, 1, 3, 2, 0),
        Player::with_values(3, 3, 2, 1, 0),
        Player::with_values(1, 5, 3, 1, 0),
        Player::with_values(2, 2, 3, 1, 0),
        Player::with_values(1, 0, 1, 1, 0).make_target(),
    ];

    const PLAYER: usize = 6;

    let previous_actions = vec![
        Action::Wall,
        Action::Barracks,
        Action::Recruit,
        Action::Recruit,
    ];

    // let previous_actions = vec![
    //     Action::Skip,
    //     Action::Barracks,
    //     Action::Barracks,
    //     Action::Recruit,
    // ];

    let state = convert_state(&previous_actions, &players, PLAYER);

    println!("{:?}", players[PLAYER]);

    let prediction = run_model(&model, &state).unwrap();
    let prediction = transform_prediction(
        &prediction,
        players[PLAYER].possible_actions(
            players.iter().enumerate().filter(|(n, _p)| *n != PLAYER),
        ),
        PLAYER
    );

    println!("{:?}\n-> {:#?}", state, prediction);
}
