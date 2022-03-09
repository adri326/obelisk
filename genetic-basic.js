import * as obelisk from "./index.js";

const N_STEPS = 40;
const MUTATION_RATE = 0.5;

const N_AGENTS = 300;
const GROUP_SIZE = 6;
const N_ROUNDS = 1000;
const SUB_ROUNDS = 200;
const RETAIN_POPULATION = Math.floor(N_AGENTS * 0.75);
const REPRODUCE_POPULATION = Math.floor(N_AGENTS * 0.25);
const SEXUATED_REPRODUCTION = true;
const TOP_AGENTS = 10;

class SimpleAgent {
    constructor(genome = null) {
        this.genome = genome ?? new Array(N_STEPS).fill(null).map(_ => {
            return ['W', 'S', 'B', 'O', 'A', 'D', 'N'][Math.floor(Math.random() * 7)];
        });
    }

    get_action(player, players, step) {
        if (this.genome[step] === 'A') {
            // try to find a player to attack
            let targets = [];

            for (let n = 0; n < players.length; n++) {
                if (players[n] === player) continue;
                let strength = players[n].walls * (players[n].defense > 0 ? 2 : 1) + players[n].soldiers;
                if (strength < player.soldiers) targets.push(n);
            }

            if (targets.length) return targets[Math.floor(Math.random() * targets.length)];
            else return ' ';
        } else if (this.genome[step] === 'N') return ' ';

        return this.genome[step];
    }

    static mutate(genome) {
        return genome.map(x => {
            if (Math.random() < MUTATION_RATE) {
                return ['W', 'S', 'B', 'O', 'A', 'D', 'N'][Math.floor(Math.random() * 7)];
            } else {
                return x;
            }
        });
    }

    static from(brain, mutate = true) {
        if (mutate) {
            return new SimpleAgent(SimpleAgent.mutate(brain.genome));
        } else {
            return new SimpleAgent(brain.genome);
        }
    }

    static breed(female, male, mutate = true) {
        let genome = female.genome.map((x, i) => Math.random() < 0.5 ? x : male.genome[i]);

        if (mutate) genome = SimpleAgent.mutate(genome);

        return new SimpleAgent(genome);
    }
}

function split_into_groups(agents, group_size) {
    agents = agents.slice();
    let groups = [];

    while (agents.length > 0) {
        let group = [];
        for (let n = 0; n < group_size && agents.length > 0; n++) {
            let index = Math.floor(Math.random() * agents.length);
            group.push(agents.splice(index, 1)[0]);
        }
        groups.push(group);
    }

    return groups;
}


function loss(player, players) {
    let max_obelisks = players.filter(p => p !== player).reduce((acc, act) => Math.max(act.obelisks, acc), 0);
    let max_barracks = players.filter(p => p !== player).reduce((acc, act) => Math.max(act.barracks, acc), 0);
    let max_soldiers = players.filter(p => p !== player).reduce((acc, act) => Math.max(act.soldiers, acc), 0);
    let max_walls = players.filter(p => p !== player).reduce((acc, act) => Math.max(act.walls, acc), 0);

    return (10 + max_obelisks) / 2 - player.obelisks
        + (max_barracks - player.barracks) / 10
        + (max_soldiers - player.soldiers) / 20
        + (max_walls - player.walls) / 10;
}

function simulate_round(agents) {
    let agent_loss = new Array(agents.length).fill(0);

    for (let sub_round = 0; sub_round < SUB_ROUNDS; sub_round++) {
        for (let group of split_into_groups(agents, GROUP_SIZE)) {
            let players = [];
            for (let n = 0; n < group.length; n++) players.push(new obelisk.Player());

            for (let step = 0; step < N_STEPS; step++) {
                let actions = group.map((agent, i) => players[i].lost() ? null : agent.get_action(players[i], players, step));
                obelisk.update(players, actions);
            }

            for (let n = 0; n < group.length; n++) {
                agent_loss[agents.indexOf(group[n])] += loss(players[n], players);
            }
        }
    }

    return agent_loss.map(x => x / SUB_ROUNDS);
}

function selection(agents, loss) {
    let sorted = [...agents.keys()].sort((a, b) => loss[a] - loss[b]).map(i => agents[i]);
    let new_agents = sorted.slice(agents.length - RETAIN_POPULATION - 1);

    while (new_agents.length < agents.length) {
        if (SEXUATED_REPRODUCTION) {
            let female = new_agents[Math.floor(Math.random() * REPRODUCE_POPULATION)];
            let male = new_agents[Math.floor(Math.random() * REPRODUCE_POPULATION)];
            new_agents.push(SimpleAgent.breed(female, male));
        } else {
            let parent = new_agents[Math.floor(Math.random() * REPRODUCE_POPULATION)];
            new_agents.push(SimpleAgent.from(parent));
        }
    }

    return new_agents;
}

let agents = new Array(N_AGENTS).fill(null).map(_ => new SimpleAgent());
for (let n = 1; n <= N_ROUNDS; n++) {
    let loss = simulate_round(agents);
    if (n % 10 === 0) {
        console.log(`=== Round ${n} ===`);
        let sorted = [...agents.keys()].sort((a, b) => loss[a] - loss[b]).map(i => agents[i]);
        for (let o = 0; o < TOP_AGENTS; o++) {
            let agent = sorted[o];
            let agent_loss = loss[agents.indexOf(agent)];
            let n_wall = agent.genome.filter(x => x === 'W').length;
            let n_soldiers = agent.genome.filter(x => x === 'S').length;
            let n_barracks = agent.genome.filter(x => x === 'B').length;
            let n_obelisks = agent.genome.filter(x => x === 'O').length;
            let n_attacks = agent.genome.filter(x => x === 'A').length;
            let n_defend = agent.genome.filter(x => x === 'D').length;
            let n_skip = agent.genome.filter(x => x === 'N').length;

            console.log(`${o.toString().padStart(3, '0')}: ${agent.genome.join("->")} | Loss: ${agent_loss.toFixed(2).padStart(5, ' ')} | W: ${n_wall.toString().padStart(2, ' ')} S: ${n_soldiers.toString().padStart(2, ' ')} B: ${n_barracks.toString().padStart(2, ' ')} O: ${n_obelisks.toString().padStart(2, ' ')} A: ${n_attacks.toString().padStart(2, ' ')} D: ${n_defend.toString().padStart(2, ' ')} X: ${n_skip.toString().padStart(2, ' ')}`);
        }
    }
    agents = selection(agents, loss);
}
