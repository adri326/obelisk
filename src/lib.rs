pub mod genetic_basic;

pub const MAX_WALLS: u8 = 10;
pub const MAX_BARRACKS: u8 = 10;
pub const MAX_OBELISKS: u8 = 10;

#[derive(Debug, Clone)]
pub struct Player {
    pub soldiers: u32,
    pub walls: u8,
    pub busy: bool,
    pub sieged: bool,
    pub defense: u8,

    pub barracks: u8,
    pub obelisks: u8,
    pub victories: usize,
    pub defeats: usize,
}

impl Player {
    pub fn new() -> Self {
        Self {
            walls: 1,
            soldiers: 1,
            barracks: 1,
            obelisks: 1,
            defense: 0,
            busy: false,
            sieged: false,
            victories: 0,
            defeats: 0
        }
    }

    pub fn with_values(walls: u8, soldiers: u32, barracks: u8, obelisks: u8, defense: u8) -> Self {
        Self {
            walls,
            soldiers,
            barracks,
            obelisks,
            defense,
            busy: false,
            sieged: false,
            victories: 0,
            defeats: 0
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
    pub fn attacked<'b, P: std::ops::DerefMut<Target=Player>>(&'b mut self, attackers: &mut [P]) {
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
            self.sieged = true;
            self.defeats += 1;
            attacker.victories += 1;
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
            res.push(Action::Barracks);
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

impl PartialEq for Player {
    fn eq(&self, other: &Player) -> bool {
        self.soldiers == other.soldiers &&
        self.walls == other.walls &&
        self.defense == other.defense &&
        self.barracks == other.barracks &&
        self.obelisks == other.obelisks
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wall,
    Recruit,
    Barracks,
    Obelisk,
    Defend,
    Attack(usize), // player
    Skip,
    None
}

impl From<Option<Action>> for Action {
    fn from(opt: Option<Action>) -> Action {
        match opt {
            Some(action) => action,
            None => Action::None
        }
    }
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

    for n in 0..players.len() {
        // SAFETY: We have `{n} ∩ attackers = ø` (from the construction of attackers)
        // We have `∀i, j, i ≠ j => attackers[i] ≠ attackers[j]` (from the construction of attackers)
        // The lifetime of attackers cannot exceed this scope (from std::mem::drop)
        // Thus `{n} ∪ attackers` is a safe partition of `players`, meaning that we can safely do `(&mut) o players[{n} ∪ attackers]`

        let mut attackers = (0..players.len())
            .filter(|&i| i != n && matches!(actions[i], Action::Attack(x) if x == n))
            .map(|i| {
                let ptr = (&mut players[i]) as *mut Player;
                unsafe {
                    &mut *ptr
                }
            })
            .collect::<Vec<_>>();

        if attackers.len() > 0 {
            debug_assert!(players[n].can_play());

            players[n].attacked(&mut attackers);
        }
        std::mem::drop(attackers);
    }

    for (n, player) in players.iter_mut().enumerate() {
        match actions[n] {
            Action::Wall if player.walls < MAX_WALLS && !player.sieged => player.walls += 1,
            Action::Barracks if player.barracks < MAX_BARRACKS && !player.sieged => player.barracks += 1,
            Action::Obelisk if player.obelisks < MAX_OBELISKS && !player.sieged => player.obelisks += 1,
            Action::Recruit => player.soldiers += player.barracks as u32,
            Action::Skip => player.soldiers += 1,
            Action::None => debug_assert!(!player.can_play()),
            _ => {}
        }

        player.busy = false; // TODO: remove busy from partialeq
        player.sieged = false;
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
            Action::Barracks,
            Action::Barracks,
            Action::Obelisk,
            Action::Barracks,
            Action::Skip,
            Action::Barracks,
            Action::Wall,
            Action::Wall,
            Action::Barracks,
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
            Action::Barracks,
            Action::Wall,
            Action::Wall,
            Action::Wall,
            Action::Wall,
            Action::Skip,
            Action::Wall,
            Action::Barracks,
            Action::Barracks,
            Action::Wall,
            Action::Barracks,
            Action::Barracks
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

    #[test]
    fn simulate_combat() {
        // Lost
        {
            let mut attacked = Player::with_values(1, 3, 1, 1, 0);
            let mut attacker = Player::with_values(1, 2, 1, 1, 0);

            attacked.attacked(&mut vec![&mut attacker]);

            assert_eq!(attacked, Player::with_values(0, 2, 1, 1, 0));
            assert_eq!(attacker, Player::with_values(1, 0, 1, 1, 0));
        }

        // Won
        {
            let mut attacked = Player::with_values(1, 3, 1, 1, 0);
            let mut attacker = Player::with_values(1, 5, 1, 1, 0);

            attacked.attacked(&mut vec![&mut attacker]);

            assert_eq!(attacked, Player::with_values(0, 0, 1, 0, 0));
            assert_eq!(attacker, Player::with_values(1, 1, 1, 2, 0));
        }

        // Draw
        {
            let mut attacked = Player::with_values(1, 3, 1, 1, 0);
            let mut attacker = Player::with_values(1, 4, 1, 1, 0);

            attacked.attacked(&mut vec![&mut attacker]);

            assert_eq!(attacked, Player::with_values(0, 0, 1, 1, 0));
            assert_eq!(attacker, Player::with_values(1, 0, 1, 1, 0));
        }

        // Two attackers: victory
        {
            let mut attacked = Player::with_values(1, 3, 1, 1, 0);
            let mut attacker_1 = Player::with_values(1, 2, 1, 1, 0);
            let mut attacker_2 = Player::with_values(1, 7, 1, 1, 0);

            attacked.attacked(&mut vec![&mut attacker_1, &mut attacker_2]);

            assert_eq!(attacked, Player::with_values(0, 0, 1, 0, 0));
            assert_eq!(attacker_1, Player::with_values(1, 0, 1, 1, 0));
            assert_eq!(attacker_2, Player::with_values(1, 1, 1, 2, 0));
        }

        // Two attackers: annihilation
        {
            let mut attacked = Player::with_values(1, 3, 1, 1, 0);
            let mut attacker_1 = Player::with_values(1, 2, 1, 1, 0);
            let mut attacker_2 = Player::with_values(1, 2, 1, 1, 0);

            attacked.attacked(&mut vec![&mut attacker_1, &mut attacker_2]);

            assert_eq!(attacked, Player::with_values(1, 3, 1, 1, 0));
            assert_eq!(attacker_1, Player::with_values(1, 0, 1, 1, 0));
            assert_eq!(attacker_2, Player::with_values(1, 0, 1, 1, 0));
        }

        // Two attackers: draw after rivalry fight
        {
            let mut attacked = Player::with_values(1, 3, 1, 1, 0);
            let mut attacker_1 = Player::with_values(1, 2, 1, 1, 0);
            let mut attacker_2 = Player::with_values(1, 6, 1, 1, 0);

            attacked.attacked(&mut vec![&mut attacker_1, &mut attacker_2]);

            assert_eq!(attacked, Player::with_values(0, 0, 1, 1, 0));
            assert_eq!(attacker_1, Player::with_values(1, 0, 1, 1, 0));
            assert_eq!(attacker_2, Player::with_values(1, 0, 1, 1, 0));
        }

        // Three attackers
        {
            let mut attacked = Player::with_values(2, 2, 2, 3, 0);
            let mut attacker_1 = Player::with_values(3, 20, 3, 2, 0);
            let mut attacker_2 = Player::with_values(2, 15, 2, 1, 0);
            let mut attacker_3 = Player::with_values(1, 13, 3, 1, 0);

            attacked.attacked(&mut vec![&mut attacker_1, &mut attacker_2, &mut attacker_3]);

            assert_eq!(attacked, Player::with_values(0, 0, 2, 2, 0));
            assert_eq!(attacker_1, Player::with_values(3, 1, 3, 3, 0));
            assert_eq!(attacker_2, Player::with_values(2, 0, 2, 1, 0));
            assert_eq!(attacker_3, Player::with_values(1, 0, 3, 1, 0));
        }
    }
}
