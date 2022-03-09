import * as obelisk from "./index.js";

function dfs(state, depth = 2) {
    if (depth === 0) {
        let loss = [];
        let max_obelisks = state.reduce((acc, act) => Math.max(act.obelisks, acc), 0);
        let max_barracks = state.reduce((acc, act) => Math.max(act.barracks, acc), 0);
        let max_soldiers = state.reduce((acc, act) => Math.max(act.soldiers, acc), 0);
        let max_walls = state.reduce((acc, act) => Math.max(act.walls, acc), 0);

        for (let n = 0; n < state.length; n++) {
            let player = state[n];
            loss.push(
                player.obelisks - max_obelisks
                + (player.barracks - max_barracks) / 10
                + (player.soldiers - max_soldiers) / 20
                + (player.walls - max_walls) / 10
            );
        }
        return [loss, []];
    }

    let loss = [];
    let best_actions = [];
    for (let n = 0; n < state.length; n++) {
        loss.push(-Infinity);
        best_actions.push(null);
    }

    for (let actions of obelisk.combine_actions(state)) {
        let next_state = obelisk.update_clone(state, actions);
        let [current_loss, next_actions] = dfs(next_state, depth - 1);

        if (current_loss.reduce((acc, act) => acc + act) >= loss.reduce((acc, act) => acc + act)) {
            best_actions = actions.map((a, i) => [a].concat(next_actions[i] ?? []));
            loss = current_loss;
        }

        // for (let n = 0; n < state.length; n++) {
        //     if (current_loss[n] >= loss[n]) {
        //         loss[n] = current_loss[n];
        //         best_actions[n] = [actions[n]].concat(next_actions[n] ?? []);
        //     }
        // }
    }

    return [loss, best_actions];
}

// let players = [
//     new obelisk.Player(2, 2, 3, 1),
//     new obelisk.Player(3, 1, 2, 2),
//     new obelisk.Player(2, 1, 2, 3)
// ];

let players = [
    // new obelisk.Player(2, 1, 2, 1), // The nameless
    // new obelisk.Player(2, 1, 2, 1), // TNW wajaeria
    new obelisk.Player(2, 1, 2, 1), // SONOC
    new obelisk.Player(2, 1, 1, 2), // New Kuiper
    // new obelisk.Player(2, 1, 2, 1), // Space Rocks
    // new obelisk.Player(1, 3, 1, 1), // SC Chonk
    // new obelisk.Player(2, 1, 2, 1), // Trars 01
    // new obelisk.Player(2, 1, 2, 1), // Golden Heights
    // new obelisk.Player(2, 1, 2, 1), // NaeNaeVille
    new obelisk.Player(2, 1, 2, 1), // Kujou Clan
    new obelisk.Player(1, 2, 2, 1), // Neo-Nuclear Empire
    new obelisk.Player(1, 2, 2, 1), // IAS
];

for (let depth = 0; depth < 10; depth++) {
    console.log(`=== Depth ${depth} ===`);
    let [loss, actions] = dfs(players, depth);
    console.log(`Loss: ${loss.map(x => x.toFixed(2)).join(" ")}`);
    console.log(`Actions: ${actions.map(a => a.join("->")).join(" ")}`);
}
