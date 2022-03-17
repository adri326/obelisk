use super::*;
use rand::Rng;

// The monte_carlo function approximates the loss of an action by running a lot of random games and averaging their results, using monte carlo's approximation
// It then returns the approximated loss and the loss variance (σ²)

pub trait AiFn<'x, R: 'x> = Fn(&'x [Player], usize, usize, &'x [Action], &'x mut R) -> Action;

pub fn mc_best_action<Ai, Loss>(
    players: &[Player],
    index: usize,
    mut constraints: Vec<(usize, Action)>,
    samples: usize,
    max_rounds: usize,
    round_offset: usize,
    ai: Ai,
    compute_loss: Loss
) -> (Action, Vec<(Action, f64, f64)>)
where
    Ai: for<'c> AiFn<'c, rand::rngs::ThreadRng> + Copy,
    Loss: for<'c> Fn(&'c [Player], usize) -> f64 + Copy,
{
    let iter = players.iter().enumerate().filter(|(n, _p)| *n != index);

    let mut best = (f64::INFINITY, Action::None);
    let mut actions = Vec::new();

    let compute_loss = move |players: &[Player]| compute_loss(players, index);

    constraints.push((index, Action::None));
    for action in players[index].possible_actions(iter) {
        if action == Action::None {
            actions.push((Action::None, f64::INFINITY, 0.0));
            continue;
        }

        *constraints.last_mut().unwrap() = (index, action);

        let (loss, variance) = monte_carlo(players, &constraints, samples, max_rounds, round_offset, ai, compute_loss);

        actions.push((action, loss, variance));

        if loss < best.0 {
            best = (loss, action);
        }
    }

    (best.1, actions)
}

pub fn monte_carlo<Ai, Loss>(
    players: &[Player],
    constraints: &[(usize, Action)],
    samples: usize,
    max_rounds: usize,
    round_offset: usize,
    ai: Ai,
    compute_loss: Loss,
) -> (f64, f64)
where
    Ai: for<'c> AiFn<'c, rand::rngs::ThreadRng> + Copy,
    Loss: for<'c> Fn(&'c [Player]) -> f64,
{
    let mut rng = rand::thread_rng();

    let mut sum = 0.0;
    let mut sum_square = 0.0; // used to compute the variance with O(1) memory

    for _n in 0..samples {
        let players = players.iter().cloned().collect::<Vec<_>>();
        let mut actions = vec![Action::Skip; players.len()];

        for n in 0..players.len() {
            actions[n] = ai(&players, n, round_offset, &[], &mut rng);
        }

        for (index, action) in constraints.iter().copied() {
            actions[index] = action;
        }

        let final_state = simulate(players, actions, ai, &mut rng, max_rounds, round_offset);
        let loss = compute_loss(&final_state);

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
    round_offset: usize,
) -> Vec<Player>
where
    R: Rng,
    Ai: for<'c> AiFn<'c, R>,
{
    players = update(players, &actions);

    let mut previous_actions = Vec::with_capacity(players.len());
    for &a in actions.iter() {
        let mut vec = Vec::with_capacity(max_rounds);
        vec.push(a);
        previous_actions.push(vec);
    }

    for round in 1..max_rounds {
        if players.iter().any(|p| p.won()) {
            break;
        }

        for n in 0..players.len() {
            let previous_actions = &mut previous_actions[n];
            actions[n] = ai(&players, n, round + round_offset, &*previous_actions, rng);
            previous_actions.push(actions[n]);
        }

        players = update(players, &actions);
    }

    players
}
