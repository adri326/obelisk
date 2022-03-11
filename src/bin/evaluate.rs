use obelisk::*;
use obelisk::monte_carlo::*;
use obelisk::genetic_basic::*;
use std::fs::File;

fn main() -> serde_json::Result<()> {
    let agents = std::fs::read_to_string("target/out.json").expect("Couldn't open target/out.json");
    let agents: Vec<SimpleAgent> = serde_json::from_str(&agents)?;

    let ai = |p: &[Player], index, round, value, rng: &mut rand::rngs::ThreadRng| {
        let agent = &agents[(value * 200.0) as usize];
        let action = agent.get_action(p, index, round, rng);
        action
    };

    let compute_loss = obelisk::genetic_basic::compute_loss;

    let players = vec![
        Player::with_values(2, 1, 3, 1, 0),
        Player::with_values(3, 1, 2, 1, 0),
        Player::with_values(3, 1, 2, 1, 0),
        Player::with_values(3, 1, 1, 2, 0),
        Player::with_values(2, 1, 3, 1, 0),
        Player::with_values(2, 1, 2, 2, 0),
        Player::with_values(2, 3, 2, 1, 0),
        Player::with_values(2, 1, 2, 2, 0),
        Player::with_values(3, 1, 2, 1, 0),
        Player::with_values(1, 2, 3, 1, 0),
        Player::with_values(2, 2, 2, 1, 0),
    ];

    let index = 9;

    println!("{:?}", mc_best_action(&players, index, 100000, 50, ai, compute_loss));

    Ok(())
}
