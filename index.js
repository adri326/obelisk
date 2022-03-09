

export class Player {
    constructor(walls = 1, soldiers = 1, barracks = 1, obelisks = 1, defense = 0) {
        this.walls = walls;
        this.soldiers = soldiers;
        this.barracks = barracks;
        this.obelisks = obelisks;
        this.defense = defense;
        this.busy = false;
    }

    // Sets all negative values to 0
    normalize() {
        this.walls = Math.max(this.walls, 0);
        this.soldiers = Math.max(this.soldiers, 0);
        this.obelisks = Math.max(this.obelisks, 0);
        this.defense = Math.max(this.defense, 0);
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

export function update(players, decisions) {
    // Step 1: update busy and defense statuses
    for (let n = 0; n < players.length; n++) {
        if (decisions[n] === 'D') {
            players[n].defense = 2;
        } else if (players[n].defense > 0) {
            players[n].defense--;
        }

        players[n].busy = decisions[n] === 'S' || typeof decisions[n] === "number";
    }

    // Step 2: resolve attacks
    for (let n = 0; n < players.length; n++) {
        let attackers = players.filter((p, i) => decisions[i] === n);
        if (attackers > 0) {
            attack(players[n], attackers);
        }
    }

    // Step 3: produce resources
    for (let n = 0; n < players.length; n++) {
        let player = players[n];
        switch (decisions[n]) {
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
    if (players[n].obelisks === 0) return null;

    let res = ['W', 'S', 'B', 'O'];
    if (players[n].walls > 0) res.push('D');

    if (players[n].soldiers > 0) {
        for (let o = 0; o < players.length; o++) {
            if (n !== o && players[o].obelisks > 0) res.push(o);
        }
    }

    return res;
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
