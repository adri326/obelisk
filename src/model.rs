use std::path::Path;
use std::hash::Hash;
use std::fmt::{Display, Debug};
use std::borrow::Borrow;
use tract_onnx::prelude::*;
use super::*;

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

pub fn convert_previous_actions(
    actions: &[Action],
    player_index: usize
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
        res[n] = categorize_action(action, player_index)
    }

    res
}

pub fn convert_player(player: &Player) -> [ModelPrec; 6] {
    [
        player.walls as ModelPrec / MAX_WALLS,
        1.0 - (-(player.soldiers as ModelPrec / SOLDIERS_SCALE)).exp(),
        player.barracks as ModelPrec / MAX_BARRACKS,
        player.obelisks as ModelPrec / MAX_OBELISKS,
        (player.defense > 0) as u8 as ModelPrec,
        (player.defense >= 2) as u8 as ModelPrec,
    ]
}

pub fn get_action_index(action: Action, player_index: usize) -> usize {
    match action {
        Action::None => 0,
        Action::Wall => 1,
        Action::Recruit => 2,
        Action::Barracks => 3,
        Action::Obelisk => 4,
        Action::Defend => 5,
        Action::Skip => 6,
        Action::Attack(n) => {
            if n < player_index {
                7 + n
            } else {
                7 + n - 1
            }
        }
    }
}

pub fn categorize_action(action: Action, player_index: usize) -> [ModelPrec; MAX_ACTIONS] {
    let mut res = [0.0; MAX_ACTIONS];

    // TODO: generate this code from actions.csv?
    let index = get_action_index(action, player_index);

    debug_assert!(index < MAX_ACTIONS);
    res[index] = 1.0;

    return res;
}

pub fn convert_state(
    previous_actions: &[Action],
    players: &[Player],
    player_index: usize
) -> [ModelPrec; INPUT_SIZE] {
    let previous_actions = convert_previous_actions(previous_actions, player_index);
    let mut res = [0.0; INPUT_SIZE];

    for (n, prev) in previous_actions.into_iter().enumerate() {
        let index = n * MAX_ACTIONS;
        let slice = &mut res[index..(index + MAX_ACTIONS)];
        slice.copy_from_slice(&prev);
    }

    for (n, player) in players.iter().enumerate() {
        let index = if n < player_index {
            (n + 1) * 6 + N_ACTIONS * MAX_ACTIONS
        } else if n == player_index {
            N_ACTIONS * MAX_ACTIONS
        } else {
            n * 6 + N_ACTIONS * MAX_ACTIONS
        };

        let slice = &mut res[index..(index+6)];
        let converted = convert_player(player);
        slice.copy_from_slice(&converted);
    }

    res
}

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

pub fn run_model(model: &Model, state: &[ModelPrec; INPUT_SIZE]) -> TractResult<Vec<ModelPrec>> {
    let tensor = Tensor::from_shape(&[1, INPUT_SIZE], state)?;

    let res = model.run(tvec!(tensor))?;

    let res: Vec<ModelPrec> = res[0].to_array_view::<ModelPrec>()?.iter().cloned().collect::<Vec<_>>();

    Ok(res)
}

pub fn transform_prediction(
    prediction: &Vec<ModelPrec>,
    actions: Vec<Action>,
    player_index: usize
) -> Vec<(Action, ModelPrec)> {
    let mut res = Vec::with_capacity(actions.len());
    let mut sum = 0.0;

    for action in actions {
        let index = get_action_index(action, player_index);

        let pred = prediction[index];
        sum += pred;
        res.push((action, pred));
    }

    for x in res.iter_mut() {
        x.1 /= sum;
    }

    res.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(&a).unwrap());

    res
}
