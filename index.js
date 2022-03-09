

export class Player {
    constructor(walls = 1, soldiers = 1, barracks = 1, obelisks = 1, defense = 0, busy = false) {
        this.walls = walls;
        this.soldiers = soldiers;
        this.barracks = barracks;
        this.obelisks = obelisks;
        this.defense = defense;
        this.busy = busy;
    }

    // Sets all negative values to 0
    normalize() {
        this.walls = Math.max(this.walls, 0);
        this.soldiers = Math.max(this.soldiers, 0);
        this.obelisks = Math.max(this.obelisks, 0);
        this.defense = Math.max(this.defense, 0);
    }

    static from(player) {
        return new Player(
            player.walls,
            player.soldiers,
            player.barracks,
            player.obelisks,
            player.defense,
            player.busy
        );
    }
}

export function attack(player, attackers) {
    attackers = attackers.filter(p => p.soldiers > 0);

    // Rivalry fights if multiple factions attack the same target: the highest player takes as damage the 2nd highest player's damage and is the only one remaining
    let sorted = attackers.sort((a, b) => b.soldiers - a.soldiers);

    if (attackers.length >= 2) {
        sorted[0].soldiers -= sorted[1].soldiers;
        for (let n = 1; n < sorted.length; n++) {
            sorted[n].soldiers = 0;
        }
    }

    let attacker = sorted[0];

    if (attacker.soldiers <= 0) return;

    // Apply walls; temporarily boost them if the defense boost is on
    let walls = player.walls * (player.defense > 0 ? 2 : 1);
    let destroyed = Math.min(walls, attacker.soldiers);
    attacker.soldiers -= destroyed;
    player.walls = Math.floor((walls - destroyed) / (player.defense > 0 ? 2 : 1));

    if (attacker.soldiers <= 0) return;

    // Last line of defense: if the army isn't busy, then soldiers mutually annihilate each other
    if (player.soldiers > 0 && !player.busy) {
        let destroyed = Math.min(player.soldiers, attacker.soldiers);
        player.soldiers -= destroyed;
        attacker.soldiers -= destroyed;
    }

    // Loot the obelisk!
    if (attacker.soldiers > 0) {
        player.obelisks--;
        attacker.obelisks++;
    }
}

export function update(players, actions) {
    // Step 1: update busy and defense statuses
    for (let n = 0; n < players.length; n++) {
        if (actions[n] === 'D') {
            players[n].defense = 2;
        } else if (players[n].defense > 0) {
            players[n].defense--;
        }

        players[n].busy = actions[n] === 'S' || typeof actions[n] === "number";
    }

    // Step 2: resolve attacks
    for (let n = 0; n < players.length; n++) {
        let attackers = players.filter((p, i) => actions[i] === n);
        if (attackers > 0) {
            attack(players[n], attackers);
        }
    }

    // Step 3: produce resources
    for (let n = 0; n < players.length; n++) {
        let player = players[n];
        switch (actions[n]) {
            case 'W':
                player.walls = Math.min(player.walls + 1, 10);
                break;
            case 'S':
                player.soldiers += player.barracks;
                break;
            case 'B':
                player.barracks = Math.min(player.barracks + 1, 10);
                break;
            case 'O':
                player.obelisks += 1;
                break;
            case ' ':
                player.soldiers += 1;
                break;
            default:
                // noop
        }
    }
}

export function clean(players) {
    for (let n = 0; n < players.length; n++) {
        let player = players[n];
        player.normalize();
        player.busy = false;
    }
}

export function possible_actions(players, n) {
    if (players[n].obelisks === 0) return [null];

    let res = ['W', 'S', 'B', 'O'];
    if (players[n].walls > 0) res.push('D');

    if (players[n].soldiers > 0) {
        for (let o = 0; o < players.length; o++) {
            if (n !== o && players[o].obelisks > 0) res.push(o);
        }
    }

    return res;
}

export function* combine_actions(players) {
    let actions = [];
    let count = [];
    for (let n = 0; n < players.length; n++) {
        actions.push(possible_actions(players, n));
        count.push(0);
    }

    while (true) {
        yield actions.map((a, i) => a[count[i]]);

        count[0]++;
        for (let n = 0; count[n] >= actions[n].length; n++) {
            count[n] = 0;
            if (n + 1 < count.length) count[n + 1]++;
            else return;
        }
    }
}

export function update_clone(players, actions) {
    let n_players = [];
    for (let n = 0; n < players.length; n++) {
        n_players.push(Player.from(players[n]));
    }
    update(n_players, actions);
    return n_players;
}

let players = [
    new Player(2, 1, 2, 1), // The nameless
    new Player(2, 1, 2, 1), // TNW wajaeria
    new Player(2, 1, 2, 1), // SONOC
    new Player(2, 1, 1, 2), // New Kuiper
    new Player(2, 1, 2, 1), // Space Rocks
    new Player(1, 3, 1, 1), // SC Chonk
    new Player(2, 1, 2, 1), // Trars 01
    new Player(2, 1, 2, 1), // Golden Heights
    new Player(2, 1, 2, 1), // NaeNaeVille
    new Player(2, 1, 2, 1), // Kujou Clan
    new Player(1, 2, 2, 1), // Neo-Nuclear Empire
    new Player(1, 2, 2, 1), // IAS
];
