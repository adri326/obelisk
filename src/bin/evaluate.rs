use obelisk::*;
use obelisk::monte_carlo::*;
// use obelisk::genetic_basic::*;
use obelisk::model::*;
#[allow(unused_imports)]
use rand::Rng;
use scoped_threadpool::Pool;
use std::time::Instant;

fn main() -> serde_json::Result<()> {
    // let agents = std::fs::read_to_string("target/out.json").expect("Couldn't open target/out.json");
    // let agents: Vec<SimpleAgent> = serde_json::from_str(&agents)?;

    // const SAMPLE_AGENTS: usize = 25000;
    // assert!(SAMPLE_AGENTS < agents.len());

    let model = load_model("target/model.onnx").unwrap();

    let ai = wrap_model(&model);
    let description = format!("weighted sample from the results of DNN gen 1");

    // let ai = |p: &[Player], index, round, _previous_actions: &[Action], rng: &mut rand::rngs::ThreadRng| {
    //     let agent = &agents[rng.gen_range(0..SAMPLE_AGENTS)];
    //     let action = agent.get_action(p, index, round, rng);
    //     action
    // };
    // let description = format!("pick random action among {}/{} agents.", SAMPLE_AGENTS, agents.len());

    let compute_loss = obelisk::genetic_basic::compute_loss;

    let players = vec![
        Player::with_values(2, 1, 4, 2, 0),
        Player::with_values(4, 1, 2, 2, 0),
        Player::with_values(3, 3, 2, 2, 0),
        Player::with_values(5, 1, 1, 2, 0),
        Player::with_values(2, 7, 3, 1, 0),
        Player::with_values(4, 1, 2, 2, 0),
        // Player::with_values(2, 0, 2, 0, 0).make_target(),
        Player::with_values(2, 4, 3, 2, 0),
        Player::with_values(3, 1, 2, 2, 0),
        Player::with_values(1, 5, 3, 2, 0),
        Player::with_values(2, 2, 3, 2, 0),
    ];

    let names = vec![
        "The Nameless",
        "TNW Wajaeria",
        "S.O.N.O.C.",
        "New Kuiper",
        "Space Rocks®",
        "Trars 01",
        // "Golden Heights",
        "NaeNaeVille",
        "Kujou Clan",
        "NN Empire",
        "I.A.S.",
    ];

    let mut pool = Pool::new(players.len() as u32);
    let samples: usize = std::env::args().last().map(|s| s.parse::<usize>().ok()).flatten().unwrap_or(1000);
    let res = std::sync::Mutex::new(Vec::new());

    let start = Instant::now();
    const TURN: usize = 4;
    let max_rounds = 50 - TURN;
    // let max_rounds = agents[0].genome.len() - TURN;

    let constraints: Vec<(usize, Action)> = vec![
        // (0, Action::Defend),
        // (4, Action::Attack(7)),
        // (8, Action::Attack(0)),
        // (4, Action::Recruit),
    ];

    pool.scoped(|scope| {
        for index in 0..players.len() {
            let players = &players;
            let res = &res;
            let constraints = constraints.clone();
            scope.execute(move || {
                let (best_action, actions) = mc_best_action(players, index, constraints, samples, max_rounds, TURN, ai, compute_loss);

                res.lock().unwrap().push((index, best_action, actions));
            });
        }
    });

    let mut res = res.into_inner().unwrap();

    res.sort_by_key(|x| x.0);

    let format_action = |action| {
        match action {
            Action::Attack(n) => print!("Attack({})", names[n]),
            x => print!("{:?}", x),
        }
    };

    println!("=== Monte Carlo Method ===");
    println!("Turn {}, players: {}", TURN + 1, players.iter().filter(|p| p.can_play()).count());
    println!("{} samples, {}.", samples, description);
    println!("Format: 'Action: loss±variance', minimize loss.");
    println!("Time taken: {:.2?}", start.elapsed());
    println!("");
    println!("== Constraints: ==");

    for (index, action) in constraints.iter().copied() {
        print!("Player {} ({}): ", names[index], index);
        format_action(action);
        println!();
    }
    println!();

    for (index, best_action, mut actions) in res {
        println!("== Player {}: {} ==", index, names[index]);
        actions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for (index2, action) in constraints.iter().copied() {
            if index2 != index {
                continue;
            }

            let (action, loss, variance) = actions.iter().find(|(a, _, _)| *a == action).unwrap();

            print!("C::> ");
            format_action(*action);
            println!(": {:.3}±{:.3}", loss, 1.96 * (variance / samples as f64).sqrt());
        }

        for (action, loss, variance) in actions.into_iter().take(6) {
            format_action(action);

            println!(": {:.3}±{:.3}", loss, 1.96 * (variance / samples as f64).sqrt());
        }


        print!("-> ");
        format_action(best_action);
        println!();

        println!();
    }

    Ok(())
}
