// Generates training data from SimpleAgents and in-training AIs
use super::genetic_basic::*;
use super::monte_carlo::*;
use super::*;
use rand::prelude::*;
use scoped_threadpool::Pool;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub struct TrainingSettings {
    pub n_data: usize,
    pub samples: usize,
    pub initial_actions: std::ops::Range<usize>,
    pub initial_noise: f64,
    pub max_rounds: usize,
    pub n_players: std::ops::Range<usize>,
    pub threads: usize,
}

impl Default for TrainingSettings {
    fn default() -> Self {
        Self {
            n_data: 10000,
            samples: 100000,
            initial_actions: 0..40,
            initial_noise: 0.2,
            max_rounds: 50,
            n_players: 4..16,
            threads: num_cpus::get(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    pub previous_actions: Vec<Vec<Action>>,
    pub players: Vec<Player>,
    pub best_actions: Vec<(Action, f64, f64)>,
}

impl TrainingData {
    pub fn new(
        previous_actions: Vec<Vec<Action>>,
        players: Vec<Player>,
        best_actions: Vec<(Action, f64, f64)>,
    ) -> Self {
        Self {
            previous_actions,
            players,
            best_actions,
        }
    }
}

pub fn generate_training_data_simpleagent(
    settings: TrainingSettings,
    agents: &[SimpleAgent],
    sample_agents: usize,
) -> Vec<TrainingData> {
    let ai = |p: &[Player], index, round, value, rng: &mut rand::rngs::ThreadRng| {
        let agent = &agents[(value * sample_agents as f64) as usize];
        let action = agent.get_action(p, index, round, rng);
        action
    };

    let compute_loss = crate::genetic_basic::compute_loss;

    generate_training_data(settings, ai, compute_loss)
}

pub fn generate_training_data<Ai, Loss>(
    settings: TrainingSettings,
    ai: Ai,
    compute_loss: Loss,
) -> Vec<TrainingData>
where
    Ai: for<'c> AiFn<'c, rand::rngs::ThreadRng> + Copy + Send,
    Loss: for<'c> Fn(&'c [Player], usize) -> f64 + Copy + Send,
{
    let res = Mutex::new(Vec::new());
    let mut pool = Pool::new(settings.threads as u32);

    pool.scoped(|scope| {
        for thread in 0..settings.threads {
            let res = &res;
            let settings = settings.clone();
            scope.execute(move || {
                let mut rng = rand::thread_rng();
                let begin = settings.n_data * thread / settings.threads;
                let end = settings.n_data * (thread + 1) / settings.threads;
                let mut tmp_res = Vec::with_capacity(end - begin);
                for data_index in begin..end {
                    if data_index % 10 == 0 {
                        println!("Thread {}: {}/{}", thread, data_index - begin, end - begin);
                    }
                    tmp_res.push(generate_training_data_sub(
                        &settings,
                        ai,
                        compute_loss,
                        &mut rng
                    ));
                }

                match res.lock() {
                    Ok(mut handle) => handle.extend(tmp_res.into_iter()),
                    Err(x) => panic!("Couldn't lock mutex! {:?}", x),
                }
            });
        }
    });

    // Filter results to remove those where the game ended
    res.into_inner()
        .unwrap()
        .into_iter()
        .filter(|data| data.best_actions.iter().any(|a| a.0 != Action::None))
        .collect::<Vec<_>>()
}

#[inline]
fn generate_training_data_sub<Ai, Loss>(
    settings: &TrainingSettings,
    ai: Ai,
    compute_loss: Loss,
    rng: &mut rand::rngs::ThreadRng,
) -> TrainingData
where
    Ai: for<'c> AiFn<'c, rand::rngs::ThreadRng> + Copy,
    Loss: for<'c> Fn(&'c [Player], usize) -> f64 + Copy,
{
    use std::cmp::Ordering;

    let mut players = vec![Player::new(); rng.gen_range(settings.n_players.clone())];
    let initial_rounds = rng.gen_range(settings.initial_actions.clone());
    let mut previous_actions = Vec::with_capacity(initial_rounds);

    for round in 0..initial_rounds {
        let actions = (0..players.len())
            .map(|n| {
                if !players[n].can_play() {
                    Action::None
                } else if rng.gen_bool(settings.initial_noise) {
                    players[n]
                        .possible_actions(
                            players.iter().enumerate().filter(|(x, _p)| *x != n),
                        )
                        .choose(rng)
                        .cloned()
                        .into()
                } else {
                    ai(&players, n, round, rng.gen(), rng)
                }
            })
            .collect::<Vec<_>>();
        players = update(players, &actions);
        previous_actions.push(actions);

        if players.iter().any(|p| p.won()) {
            break;
        }
    }

    let mut best_actions = Vec::with_capacity(players.len());

    for player_index in 0..players.len() {
        let (best, mut losses) = mc_best_action(
            &players,
            player_index,
            settings.samples,
            settings.max_rounds - initial_rounds,
            initial_rounds,
            ai,
            compute_loss,
        );

        losses.sort_by(|(_, a, _), (_, b, _)| a.partial_cmp(&b).unwrap_or(Ordering::Equal));

        let (_, loss, variance) = losses[0];
        let confidence = 1.96 * (variance / settings.samples as f64).sqrt();

        best_actions.push((best, loss, confidence));
    }
    TrainingData::new(previous_actions, players, best_actions)
}
