use std::path::Path;
use std::hash::Hash;
use std::fmt::{Display, Debug};
use std::borrow::Borrow;
use tract_onnx::prelude::*;
use super::*;
use super::monte_carlo::AiFn;

pub const N_ACTIONS: usize = 8;
pub const MAX_PLAYERS: usize = 16;
pub const ACTION_ATTACK: usize = 7;
pub const MAX_ACTIONS: usize = ACTION_ATTACK + MAX_PLAYERS - 1;
pub const INPUT_SIZE: usize = MAX_ACTIONS * N_ACTIONS + 6 * MAX_PLAYERS;

pub type ModelPrec = f32;
const DATUM_PREC: DatumType = DatumType::F32;

pub const MAX_WALLS: ModelPrec = 10.0;
pub const MAX_BARRACKS: ModelPrec = 10.0;
pub const MAX_OBELISKS: ModelPrec = 10.0;
pub const SOLDIERS_SCALE: ModelPrec = 5.0;

fn convert_previous_actions(
    actions: &[Action],
    inverse_permutation: &[usize],
) -> [[ModelPrec; MAX_ACTIONS]; N_ACTIONS] {
    let mut res = [[0.0; MAX_ACTIONS]; N_ACTIONS];

    for (n, action) in actions
        .iter()
        .copied()
        .rev()
        .chain([Action::None].into_iter().cycle())
        .take(N_ACTIONS)
        .enumerate()
    {
        res[n] = categorize_action(action, inverse_permutation)
    }

    res
}

fn convert_player(player: &Player) -> [ModelPrec; 6] {
    [
        player.walls as ModelPrec / MAX_WALLS,
        1.0 - (-(player.soldiers as ModelPrec / SOLDIERS_SCALE)).exp(),
        player.barracks as ModelPrec / MAX_BARRACKS,
        player.obelisks as ModelPrec / MAX_OBELISKS,
        (player.defense > 0) as u8 as ModelPrec,
        (player.defense >= 2) as u8 as ModelPrec,
    ]
}

fn get_action_index(action: Action, inverse_permutation: &[usize]) -> usize {
    match action {
        Action::None => 0,
        Action::Wall => 1,
        Action::Recruit => 2,
        Action::Barracks => 3,
        Action::Obelisk => 4,
        Action::Defend => 5,
        Action::Skip => 6,
        Action::Attack(n) => {
            7 + inverse_permutation[n]
            // if n < player_index {
            //     7 + n
            // } else {
            //     7 + n - 1
            // }
        }
    }
}

fn categorize_action(action: Action, inverse_permutation: &[usize]) -> [ModelPrec; MAX_ACTIONS] {
    let mut res = [0.0; MAX_ACTIONS];

    // TODO: generate this code from actions.csv?
    let index = get_action_index(action, inverse_permutation);

    debug_assert!(index < MAX_ACTIONS);
    res[index] = 1.0;

    return res;
}

#[inline]
fn compute_permutation(players: &[Player], player_index: usize) -> (Vec<usize>, Vec<usize>) {
    let mut permutation = Vec::with_capacity(players.len());
    permutation.push(player_index);
    for n in 0..players.len() {
        if n == player_index {
            continue
        } else {
            permutation.push(n);
        }
    }

    permutation[1..].sort_by(|&a, &b| {
        let a = players[a].walls as u32 * (if players[a].defense > 0 {2} else {1}) as u32 + players[a].soldiers;
        let b = players[b].walls as u32 * (if players[b].defense > 0 {2} else {1}) as u32 + players[b].soldiers;

        b.partial_cmp(&a).unwrap()
    });

    let mut inverse_permutation = vec![0; players.len()];
    for (n, p) in permutation.iter().copied().enumerate() {
        inverse_permutation[p] = n;
    }

    (permutation, inverse_permutation)
}

#[inline]
pub fn run_model(
    model: &Model,
    previous_actions: &[Action],
    players: &[Player],
    player_index: usize,
    actions: &[Action],
) -> TractResult<Vec<(Action, ModelPrec)>> {
    let (permutation, inverse_permutation) = compute_permutation(players, player_index);

    let previous_actions = convert_previous_actions(previous_actions, &inverse_permutation);
    let mut input = [0.0; INPUT_SIZE];

    for (n, prev) in previous_actions.into_iter().enumerate() {
        let index = n * MAX_ACTIONS;
        let slice = &mut input[index..(index + MAX_ACTIONS)];
        slice.copy_from_slice(&prev);
    }

    for (n, player) in players.iter().enumerate() {
        let index = permutation[n] * 6 + N_ACTIONS * MAX_ACTIONS;

        let slice = &mut input[index..(index+6)];
        let converted = convert_player(player);
        slice.copy_from_slice(&converted);
    }

    let tensor = Tensor::from_shape(&[1, INPUT_SIZE], &input)?;

    let prediction = model.run(tvec!(tensor))?;

    let prediction: Vec<ModelPrec> = prediction[0].to_array_view::<ModelPrec>()?.iter().cloned().collect::<Vec<_>>();

    // println!("{:?}", prediction);

    let mut res = Vec::with_capacity(actions.len());
    let mut sum = 0.0;

    for action in actions.iter().copied() {
        let index = get_action_index(action, &inverse_permutation);
        res.push((action, prediction[index]));
        sum += prediction[index];
    }

    for x in res.iter_mut() {
        x.1 /= sum;
    }

    res.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal));

    Ok(res)
}

// Workaround for issue https://github.com/rust-lang/rust/issues/55997
mod workaround_55997 {
    use super::*;

    pub type ModelFact = impl Fact + Hash + Clone + 'static;
    pub type ModelOp = impl Debug + Display + AsRef<dyn Op + 'static> + AsMut<dyn Op + 'static> + Clone + Hash + 'static;
    pub type ModelGraph = impl Borrow<Graph<ModelFact, ModelOp>> + Hash;
    pub type Model = SimplePlan<ModelFact, ModelOp, ModelGraph>;

    pub fn load_model(path: impl AsRef<Path>) -> TractResult<Model> {
        // debug_assert!(INPUT_SIZE == 272);
        tract_onnx::onnx()
            .model_for_path(path)?
            .with_input_fact(0, InferenceFact::dt_shape(
                DATUM_PREC,
                &[1, INPUT_SIZE],
            ))?
            .with_output_fact(0, InferenceFact::dt_shape(
                DATUM_PREC,
                &[1, MAX_ACTIONS]
            ))?
            .into_optimized()?
            .into_runnable()
    }
}
pub use workaround_55997::*;

pub type ModelFn<'a> = impl 'a + Copy + Send + (for<'c> AiFn<'c, rand::rngs::ThreadRng>);

pub fn wrap_model<'a>(model: &'a Model) -> ModelFn<'a> {
    use rand::Rng;
    move |players: &[Player], index: usize, _round: usize, previous_actions: &[Action], rng: &mut rand::rngs::ThreadRng| {
        let possible_actions = players[index].possible_actions(
            players.iter().enumerate().filter(|(x, _p)| *x != index),
        );

        let predictions = run_model(
            model,
            previous_actions,
            players,
            index,
            &possible_actions
        ).unwrap();

        let best_action = predictions[0].0;

        let choice = rng.gen::<ModelPrec>();
        let mut sum = 0.0;
        for (action, prob) in predictions {
            sum += prob;
            if sum > choice {
                return action
            }
        }

        best_action
    }
}
