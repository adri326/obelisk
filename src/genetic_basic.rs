use super::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimpleAgentAction {
    Wall,
    Recruit,
    Barracks,
    Obelisk,
    Attack,
    Defend,
    Skip,
}

impl SimpleAgentAction {
    #[inline]
    pub fn rand(rng: &mut impl Rng) -> Self {
        [
            SimpleAgentAction::Wall,
            SimpleAgentAction::Recruit,
            SimpleAgentAction::Barracks,
            SimpleAgentAction::Obelisk,
            SimpleAgentAction::Attack,
            SimpleAgentAction::Defend,
            SimpleAgentAction::Skip,
        ][rng.gen_range(0..7)]
    }
}

impl From<SimpleAgentAction> for Action {
    fn from(action: SimpleAgentAction) -> Action {
        match action {
            SimpleAgentAction::Wall => Action::Wall,
            SimpleAgentAction::Recruit => Action::Recruit,
            SimpleAgentAction::Barracks => Action::Barracks,
            SimpleAgentAction::Obelisk => Action::Obelisk,
            SimpleAgentAction::Attack => Action::None,
            SimpleAgentAction::Defend => Action::Defend,
            SimpleAgentAction::Skip => Action::Skip,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SimpleAgent {
    pub genome: Vec<SimpleAgentAction>,
}

impl SimpleAgent {
    pub fn new(steps: usize) -> Self {
        let mut genome = Vec::with_capacity(steps);
        let mut rng = rand::thread_rng();

        for _n in 0..steps {
            genome.push(SimpleAgentAction::rand(&mut rng));
        }

        Self { genome }
    }

    pub fn from_rng(steps: usize, rng: &mut impl Rng) -> Self {
        let mut genome = Vec::with_capacity(steps);

        for _n in 0..steps {
            genome.push(SimpleAgentAction::rand(rng));
        }

        Self { genome }
    }

    pub fn mutate(&self, mutation: f64) -> Self {
        let mut new_genome = self.genome.clone();

        let mut rng = rand::thread_rng();

        for n in 0..self.genome.len() {
            if rng.gen_bool(mutation) {
                new_genome[n] = SimpleAgentAction::rand(&mut rng);
            }
        }

        Self { genome: new_genome }
    }

    pub fn breed(&self, partner: &SimpleAgent, mutation: f64) -> Self {
        let mut new_genome = self.genome.clone();

        let mut rng = rand::thread_rng();

        for n in 0..self.genome.len() {
            if n < partner.genome.len() && rng.gen_bool(0.5) {
                new_genome[n] = partner.genome[n];
            }

            if rng.gen_bool(mutation) {
                new_genome[n] = SimpleAgentAction::rand(&mut rng);
            }
        }

        Self { genome: new_genome }
    }

    pub fn irradiate(&mut self, radiation: f64) {
        let mut rng = rand::thread_rng();

        for n in 0..self.genome.len() {
            if rng.gen_bool(radiation) {
                self.genome[n] = SimpleAgentAction::rand(&mut rng);
            }
        }
    }

    #[inline]
    pub fn get_action<'b>(
        &'b self,
        players: &'b [Player],
        index: usize,
        step: usize,
        rng: &'b mut impl Rng,
    ) -> Action {
        if !players[index].can_play() {
            return Action::None;
        }
        if step >= self.genome.len() {
            return Action::Skip;
        }

        if self.genome[step] == SimpleAgentAction::Attack {
            let targets: SmallVec<[usize; 10]> = players
                .iter()
                .enumerate()
                .filter(|&(n, p)| {
                    let strength = p.walls as u32 * if p.defense > 0 { 2 } else { 1 } + p.soldiers;
                    return n != index && strength < players[index].soldiers && p.can_be_attacked();
                })
                .map(|(n, _p)| n)
                .collect::<SmallVec<_>>();

            if targets.len() > 0 {
                return Action::Attack(*targets.choose(rng).unwrap());
            } else {
                return Action::Skip;
            }
        }

        return self.genome[step].into();
    }
}

impl std::fmt::Display for SimpleAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use SimpleAgentAction::*;

        let mut first = true;
        for gene in self.genome.iter() {
            if first {
                first = false;
            } else {
                write!(f, "→")?;
            }

            write!(
                f,
                "{}",
                match gene {
                    Wall => "W",
                    Recruit => "S",
                    Barracks => "B",
                    Obelisk => "O",
                    Attack => "A",
                    Defend => "D",
                    Skip => "N",
                }
            )?;
        }

        fn count(genome: &[SimpleAgentAction], target: SimpleAgentAction) -> usize {
            genome.iter().filter(|g| **g == target).count()
        }

        write!(
            f,
            " (W: {:2}, S: {:2}, B: {:2}, O: {:2}, A: {:2}, D: {:2}, N: {:2})",
            count(&self.genome, Wall),
            count(&self.genome, Recruit),
            count(&self.genome, Barracks),
            count(&self.genome, Obelisk),
            count(&self.genome, Attack),
            count(&self.genome, Defend),
            count(&self.genome, Skip),
        )
    }
}

pub fn compute_loss(players: &[Player], index: usize) -> f64 {
    let iter = players.iter().enumerate().filter(|(n, _p)| *n != index);

    let (max_obelisks, max_barracks, max_soldiers, max_walls) = iter
        .map(|(_, x)| {
            (
                x.obelisks as f64,
                x.barracks as f64,
                x.soldiers as f64,
                x.walls as f64,
            )
        })
        .reduce(|acc, act| {
            (
                acc.0.max(act.0),
                acc.1.max(act.1),
                acc.2.max(act.2),
                acc.3.max(act.3),
            )
        })
        .unwrap_or((0.0, 0.0, 0.0, 0.0));

    let player = &players[index];

    let mut res = (10.0 + max_obelisks) / 2.0 - player.obelisks as f64
        + (max_barracks - player.barracks as f64) / 5.0
        + (max_soldiers - player.soldiers as f64) / 10.0
        + (max_walls - player.walls as f64) / 5.0;

    if player.obelisks == 0 {
        res += 4.0;
    }

    if max_obelisks == 10.0 {
        res += 2.0; // penalty for not winning the game first
    }

    res += player.defeats as f64 * 0.5; // penalty for losing sieges
    res += (player.victories as f64 * -0.5).exp() * 4.0; // bonus for winning sieges

    res
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimulationSettings {
    pub sub_rounds: usize,
    pub group_size: usize,
    pub n_steps: usize,

    pub population: usize,
    pub retain_population: usize,
    pub reproduce_population: usize,
    pub new_population: usize,
    pub mutation: f64,
    pub radiation: f64,
    pub sexuated_reproduction: bool,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            sub_rounds: 100,
            group_size: 12,
            n_steps: 50,

            population: 1000,
            retain_population: 500,
            reproduce_population: 250,
            new_population: 50,
            mutation: 0.02,
            radiation: 0.002,
            sexuated_reproduction: true,
        }
    }
}

// TODO: optimize this further
pub fn simulate_round(agents: &[SimpleAgent], settings: SimulationSettings) -> Vec<f64> {
    let mut loss = vec![0.0; agents.len()];

    let mut rng = rand::thread_rng();
    let mut agents_ref = agents.iter().enumerate().collect::<Vec<_>>();

    for _sub_round in 0..settings.sub_rounds {
        agents_ref.shuffle(&mut rng);
        for group in agents_ref.chunks(settings.group_size) {
            let mut players = vec![Player::new(); group.len()];

            for step in 0..settings.n_steps {
                // Collect the actions of each agent
                let actions = group
                    .iter()
                    .enumerate()
                    .map(|(i, (_, agent))| agent.get_action(&players, i, step, &mut rng))
                    .collect::<Vec<_>>();

                players = update(players, &actions);

                if players.iter().any(|p| p.won()) {
                    break;
                }
            }

            // Compute loss
            for (n, (i, _agent)) in group.into_iter().enumerate() {
                let tmp = compute_loss(&players, n);
                loss[*i] += tmp * tmp;
            }
        }
    }

    for x in loss.iter_mut() {
        *x /= settings.sub_rounds as f64;
    }

    loss
}

pub fn selection(
    agents: Vec<SimpleAgent>,
    loss: Vec<f64>,
    settings: SimulationSettings,
) -> Vec<SimpleAgent> {
    let mut agents = agents.into_iter().enumerate().collect::<Vec<_>>();
    agents.sort_unstable_by(|(n1, _), (n2, _)| loss[*n1].partial_cmp(&loss[*n2]).unwrap());
    let mut agents = agents.into_iter().map(|(_, a)| a).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();

    for n in 0..settings.retain_population {
        agents[n].irradiate(settings.radiation);
    }

    for n in settings.retain_population..agents.len() {
        if n < settings.retain_population + settings.new_population {
            agents[n] = SimpleAgent::from_rng(settings.n_steps, &mut rng);
        } else if settings.sexuated_reproduction {
            let new_agent = {
                let female = agents[0..settings.reproduce_population]
                    .choose(&mut rng)
                    .unwrap();
                let male = agents[0..settings.reproduce_population]
                    .choose(&mut rng)
                    .unwrap();
                female.breed(male, settings.mutation)
            };
            agents[n] = new_agent;
        } else {
            let new_agent = agents[0..settings.reproduce_population]
                .choose(&mut rng)
                .unwrap()
                .mutate(settings.mutation);

            agents[n] = new_agent;
        }
    }

    agents
}

pub fn new_agents(settings: SimulationSettings) -> Vec<SimpleAgent> {
    let mut res = Vec::with_capacity(settings.population);

    let mut rng = rand::thread_rng();

    for _n in 0..settings.population {
        res.push(SimpleAgent::from_rng(settings.n_steps, &mut rng));
    }

    res
}

pub fn print_best(agents: &Vec<SimpleAgent>, loss: &Vec<f64>, best_n: usize) {
    let mut agents = agents.iter().enumerate().collect::<Vec<_>>();
    agents.sort_unstable_by(|(n1, _), (n2, _)| loss[*n1].partial_cmp(&loss[*n2]).unwrap());

    for (index, agent) in agents.iter().take(best_n) {
        println!("{:05}: {} | Loss: {:.3}", index, agent, loss[*index]);
    }
}
