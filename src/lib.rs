use std::cell::RefCell;

pub const MAX_WALLS: u8 = 10;
pub const MAX_BARRACKS: u8 = 10;
pub const MAX_OBELISKS: u8 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub soldiers: u32,
    pub walls: u8,
    pub busy: bool,
    pub defense: u8,

    pub barracks: u8,
    pub obelisks: u8,
}

impl Player {
    pub fn new() -> Self {
        Self {
            walls: 1,
            soldiers: 1,
            barracks: 1,
            obelisks: 1,
            busy: false,
            defense: 0
        }
    }

    pub fn with_values(walls: u8, soldiers: u32, barracks: u8, obelisks: u8, defense: u8) -> Self {
        Self {
            walls,
            soldiers,
            barracks,
            obelisks,
            defense,
            busy: false
        }
    }

    #[inline(always)]
    pub fn lost(&self) -> bool {
        return self.obelisks == 0;
    }

    #[inline(always)]
    pub fn won(&self) -> bool {
        return self.obelisks == 10;
    }

    #[inline(always)]
    pub fn can_play(&self) -> bool {
        return !self.lost() && !self.won();
    }

    #[inline]
    pub fn attacked<'b, P: std::ops::DerefMut<Target=Player>>(&'b mut self, mut attackers: Vec<P>) {
        // TODO: use copies of the values to make it easier for LLVM to optimize this away
        debug_assert!(attackers.iter().all(|p| p.soldiers > 0));

        if attackers.len() >= 2 {
            attackers.sort_unstable_by_key(|p| -(p.soldiers as i32));

            attackers[0].soldiers -= attackers[1].soldiers;
            for p in attackers.iter_mut().skip(1) {
                p.soldiers = 0;
            }
        }

        let attacker = &mut attackers[0];

        if attacker.soldiers == 0 {
            return
        }

        let walls: u32 = self.walls as u32 * if self.defense > 0 { 2 } else { 1 };
        if attacker.soldiers <= walls {
            self.walls = ((walls - attacker.soldiers) / if self.defense > 0 { 2 } else { 1 }) as u8;
            attacker.soldiers = 0;
            return
        } else {
            attacker.soldiers -= walls;
            self.walls = 0;
        }

        if self.soldiers > 0 && !self.busy {
            let destroyed = self.soldiers.min(attacker.soldiers);
            self.soldiers -= destroyed;
            attacker.soldiers -= destroyed;
        }

        if attacker.soldiers > 0 {
            self.obelisks -= 1;
            attacker.obelisks += 1;
        }
    }

    #[inline]
    pub fn possible_actions<'b, I: Iterator<Item=(usize, &'b Player)>>(&self, players: I) -> Vec<Action> {
        if !self.can_play() {
            return vec![Action::None];
        }

        let mut res = Vec::with_capacity(6);
        res.push(Action::Recruit);
        res.push(Action::Skip);
        if self.walls < MAX_WALLS {
            res.push(Action::Wall);
        }
        if self.walls > 0 {
            res.push(Action::Defend);
        }

        if self.barracks < MAX_BARRACKS {
            res.push(Action::Barrack);
        }

        if self.obelisks < MAX_OBELISKS {
            res.push(Action::Obelisk);
        }

        if self.soldiers > 0 {
            for (n, player) in players {
                if player.can_play() {
                    res.push(Action::Attack(n));
                }
            }
        }

        res
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wall,
    Recruit,
    Barrack,
    Obelisk,
    Defend,
    Attack(usize), // player
    Skip,
    None
}

#[inline]
pub fn update(mut players: Vec<Player>, actions: &[Action]) -> Vec<Player> {
    debug_assert!(players.len() == actions.len());

    for (n, player) in players.iter_mut().enumerate() {
        if actions[n] == Action::Defend {
            player.defense = 2;
        } else if player.defense > 0 {
            player.defense -= 1;
        }

        player.busy = matches!(actions[n], Action::Attack(_) | Action::Recruit);
    }

    let players = players.into_iter().map(|p| RefCell::new(p)).collect::<Vec<_>>();

    for n in 0..players.len() {
        let attackers = (0..players.len()).filter(|i| matches!(actions[*i], Action::Attack(x) if x == n)).collect::<Vec<_>>();
        if attackers.len() > 0 {
            debug_assert!(!attackers.iter().any(|i| *i == n));
            debug_assert!(players[n].borrow().can_play());
            players[n].borrow_mut().attacked(attackers.into_iter().map(|i| players[i].borrow_mut()).collect::<Vec<_>>());
        }
    }

    let mut players = players.into_iter().map(|r| r.into_inner()).collect::<Vec<_>>();

    for (n, player) in players.iter_mut().enumerate() {
        match actions[n] {
            Action::Wall if player.walls < MAX_WALLS => player.walls += 1,
            Action::Barrack if player.barracks < MAX_BARRACKS => player.barracks += 1,
            Action::Obelisk if player.obelisks < MAX_OBELISKS => player.obelisks += 1,
            Action::Recruit => player.soldiers += player.barracks as u32,
            Action::Skip => player.soldiers += 1,
            Action::None => debug_assert!(!player.can_play()),
            _ => {}
        }

        player.busy = false; // TODO: remove busy from partialeq
    }

    players
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_update() {
        let mut state = vec![Player::new(); 12];

        let decisions_0 = [
            Action::Wall,
            Action::Barrack,
            Action::Barrack,
            Action::Obelisk,
            Action::Barrack,
            Action::Skip,
            Action::Barrack,
            Action::Wall,
            Action::Wall,
            Action::Barrack,
            Action::Skip,
            Action::Skip
        ];

        state = update(state, &decisions_0);

        assert_eq!(state, vec![
            Player::with_values(2, 1, 1, 1, 0),
            Player::with_values(1, 1, 2, 1, 0),
            Player::with_values(1, 1, 2, 1, 0),
            Player::with_values(1, 1, 1, 2, 0),
            Player::with_values(1, 1, 2, 1, 0),
            Player::with_values(1, 2, 1, 1, 0),
            Player::with_values(1, 1, 2, 1, 0),
            Player::with_values(2, 1, 1, 1, 0),
            Player::with_values(2, 1, 1, 1, 0),
            Player::with_values(1, 1, 2, 1, 0),
            Player::with_values(1, 2, 1, 1, 0),
            Player::with_values(1, 2, 1, 1, 0),
        ]);

        let decisions_1 = [
            Action::Barrack,
            Action::Wall,
            Action::Wall,
            Action::Wall,
            Action::Wall,
            Action::Skip,
            Action::Wall,
            Action::Barrack,
            Action::Barrack,
            Action::Wall,
            Action::Barrack,
            Action::Barrack
        ];

        state = update(state, &decisions_1);

        assert_eq!(state, vec![
            Player::with_values(2, 1, 2, 1, 0),
            Player::with_values(2, 1, 2, 1, 0),
            Player::with_values(2, 1, 2, 1, 0),
            Player::with_values(2, 1, 1, 2, 0),
            Player::with_values(2, 1, 2, 1, 0),
            Player::with_values(1, 3, 1, 1, 0),
            Player::with_values(2, 1, 2, 1, 0),
            Player::with_values(2, 1, 2, 1, 0),
            Player::with_values(2, 1, 2, 1, 0),
            Player::with_values(2, 1, 2, 1, 0),
            Player::with_values(1, 2, 2, 1, 0),
            Player::with_values(1, 2, 2, 1, 0),
        ]);
    }
}