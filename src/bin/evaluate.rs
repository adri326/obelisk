use obelisk::*;
use obelisk::monte_carlo::*;
use obelisk::genetic_basic::*;
use scoped_threadpool::Pool;
use std::time::Instant;

fn main() -> serde_json::Result<()> {
    let agents = std::fs::read_to_string("target/out.json").expect("Couldn't open target/out.json");
    let agents: Vec<SimpleAgent> = serde_json::from_str(&agents)?;

    const SAMPLE_AGENTS: usize = 25000;
    assert!(SAMPLE_AGENTS < agents.len());

    let ai = |p: &[Player], index, round, value, rng: &mut rand::rngs::ThreadRng| {
        let agent = &agents[(value * SAMPLE_AGENTS as f64) as usize];
        let action = agent.get_action(p, index, round, rng);
        action
    };

    let compute_loss = obelisk::genetic_basic::compute_loss;

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

    let names = vec![
        "The Nameless",
        "TNW Wajaeria",
        "S.O.N.O.C.",
        "New Kuiper",
        "Space Rocks®",
        "Trars 01",
        "Golden Heights",
        "NaeNaeVille",
        "Kujou Clan",
        "NN Empire",
        "I.A.S.",
        "SC CHONK"
    ];

    let mut pool = Pool::new(players.len() as u32);
    const SAMPLES: usize = 100000;
    let res = std::sync::Mutex::new(Vec::new());

    let start = Instant::now();
    const TURN: usize = 3;
    let max_rounds = agents[0].genome.len() - TURN;

    pool.scoped(|scope| {
        for index in 0..players.len() {
            let players = &players;
            let res = &res;
            scope.execute(move || {
                let (best_action, actions) = mc_best_action(players, index, SAMPLES, max_rounds, TURN, ai, compute_loss);

                res.lock().unwrap().push((index, best_action, actions));
            });
        }
    });

    let mut res = res.into_inner().unwrap();

    res.sort_by_key(|x| x.0);

    println!("=== Monte Carlo Method ===");
    println!("Turn {}, players: {}", TURN + 1, players.iter().filter(|p| p.can_play()).count());
    println!("{} samples, pick random action among {}/{} agents.", SAMPLES, SAMPLE_AGENTS, agents.len());
    println!("Format: 'Action: loss±variance', minimize loss.");
    println!("Time taken: {:.2?}", start.elapsed());
    println!("");

    for (index, best_action, mut actions) in res {
        println!("== Player {}: {} ==", index, names[index]);
        actions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        for (action, loss, variance) in actions.into_iter().take(6) {
            match action {
                Action::Attack(n) => print!("Attack({})", names[n]),
                x => print!("{:?}", x),
            };

            println!(": {:.3}±{:.3}", loss, 1.96 * (variance / SAMPLES as f64).sqrt());
        }

        match best_action {
            Action::Attack(n) => println!("-> Attack({})", names[n]),
            x => println!("-> {:?}", x),
        };

        println!();
    }

    Ok(())
}
