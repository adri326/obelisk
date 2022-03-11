use super::*;
use rand::Rng;

// The monte_carlo function approximates the loss of an action by running a lot of random games and averaging their results, using monte carlo's approximation
// It then returns the approximated loss and the loss variance (σ²)

pub trait AiFn<'x, R: 'x> = Fn(&'x [Player], usize, usize, f64, &'x mut R) -> Action;

pub fn mc_best_action<Ai, Loss>(
    players: &[Player],
    index: usize,
    samples: usize,
    max_rounds: usize,
    ai: Ai,
    compute_loss: Loss
) -> Action
where
    Ai: for<'c> AiFn<'c, rand::rngs::ThreadRng> + Copy,
    Loss: for<'c> Fn(&'c [Player], usize) -> f64 + Copy,
{
    let iter = players.iter().enumerate().filter(|(n, _p)| *n != index);

    let mut best = (f64::INFINITY, Action::None);
    for action in players[index].possible_actions(iter) {
        let (loss, variance) = monte_carlo(players, index, action, samples, max_rounds, ai, compute_loss);

        println!("{:?}: {:.3}", action, loss);

        if loss < best.0 {
            best = (loss, action);
        }
    }

    best.1
}

pub fn monte_carlo<Ai, Loss>(
    players: &[Player],
    index: usize,
    action: Action,
    samples: usize,
    max_rounds: usize,
    ai: Ai,
    compute_loss: Loss,
) -> (f64, f64)
where
    Ai: for<'c> AiFn<'c, rand::rngs::ThreadRng> + Copy,
    Loss: for<'c> Fn(&'c [Player], usize) -> f64,
{
    let mut rng = rand::thread_rng();

    let mut sum = 0.0;
    let mut sum_square = 0.0; // used to compute the variance with O(1) memory

    for _n in 0..samples {
        let players = players.iter().cloned().collect::<Vec<_>>();
        let mut actions = vec![Action::Skip; players.len()];

        actions[index] = action;

        for n in 0..players.len() {
            if n == index {
                continue;
            }

            let value: f64 = rng.gen();
            actions[n] = ai(&players, n, 0, value, &mut rng);
        }

        let final_state = simulate(players, actions, ai, &mut rng, max_rounds);
        let loss = compute_loss(&final_state, index);

        sum += loss;
        sum_square += loss * loss;
    }

    sum /= samples as f64;
    sum_square /= samples as f64;

    (sum, sum_square - sum * sum)
}

#[inline]
fn simulate<Ai, R>(
    mut players: Vec<Player>,
    mut actions: Vec<Action>,
    ai: Ai,
    rng: &mut R,
    max_rounds: usize,
) -> Vec<Player>
where
    R: Rng,
    Ai: for<'c> AiFn<'c, R>,
{
    players = update(players, &actions);

    for round in 1..max_rounds {
        if players.iter().any(|p| p.won()) {
            break;
        }

        for n in 0..players.len() {
            let value: f64 = rng.gen();
            actions[n] = ai(&players, n, round, value, rng);
        }

        players = update(players, &actions);
    }

    players
}