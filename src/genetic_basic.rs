use super::*;
use rand::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimpleAgentAction {
    Wall,
    Recruit,
    Barracks,
    Obelisk,
    Attack,
    Defend,
    Skip
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

        Self {
            genome
        }
    }

    pub fn mutate(&self, mutation: f64) -> Self {
        let mut new_genome = self.genome.clone();

        let mut rng = rand::thread_rng();

        for n in 0..self.genome.len() {
            if rng.gen_bool(mutation) {
                new_genome[n] = SimpleAgentAction::rand(&mut rng);
            }
        }

        Self {
            genome: new_genome
        }
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

        Self {
            genome: new_genome
        }
    }

    pub fn get_action(&self, players: &[Player], index: usize, step: usize, rng: &mut impl Rng) -> Action {
        if step >= self.genome.len() || !players[index].can_play() {
            return Action::None;
        }

        if self.genome[step] == SimpleAgentAction::Attack {
            let targets = players.iter().enumerate().filter(|&(n, p)| {
                let strength = p.walls as u32 * if p.defense > 0 {2} else {1} + p.soldiers;
                return n != index && strength < players[index].soldiers
            }).map(|(n, _p)| n).collect::<Vec<_>>();

            if targets.len() > 0 {
                return Action::Attack(*targets.choose(rng).unwrap());
            } else {
                return Action::Skip;
            }
        }

        return self.genome[step].into();
    }
}
