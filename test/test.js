import assert from "node:assert";
import {Player, update, clean, attack} from "../index.js";
import permutations from "just-permutations";

describe("update", () => {
    it("Should successfully simulate the first turns of the game", () => {
        let state = new Array(12).fill(null).map(_ => new Player());
        let decisions_0 = ['W', 'B', 'B', 'O', 'B', ' ', 'B', 'W', 'W', 'B', ' ', ' '];

        let turn_1 = [
            new Player(2, 1, 1, 1),
            new Player(1, 1, 2, 1),
            new Player(1, 1, 2, 1),
            new Player(1, 1, 1, 2),
            new Player(1, 1, 2, 1),
            new Player(1, 2, 1, 1),
            new Player(1, 1, 2, 1),
            new Player(2, 1, 1, 1),
            new Player(2, 1, 1, 1),
            new Player(1, 1, 2, 1),
            new Player(1, 2, 1, 1),
            new Player(1, 2, 1, 1),
        ];
        let decisions_1 = ['B', 'W', 'W', 'W', 'W', ' ', 'W', 'B', 'B', 'W', 'B', 'B'];

        let turn_2 = [
            new Player(2, 1, 2, 1),
            new Player(2, 1, 2, 1),
            new Player(2, 1, 2, 1),
            new Player(2, 1, 1, 2),
            new Player(2, 1, 2, 1),
            new Player(1, 3, 1, 1),
            new Player(2, 1, 2, 1),
            new Player(2, 1, 2, 1),
            new Player(2, 1, 2, 1),
            new Player(2, 1, 2, 1),
            new Player(1, 2, 2, 1),
            new Player(1, 2, 2, 1),
        ];


        update(state, decisions_0);
        clean(state);
        assert.deepEqual(state, turn_1);

        update(state, decisions_1);
        clean(state);
        assert.deepEqual(state, turn_2);
    });

    it("Should simulate a non-succeeding combat round", () => {
        let attacked = new Player(1, 3, 1, 1);
        let attacker = new Player(1, 2, 1, 1);
        attack(attacked, [attacker]);
        assert.deepEqual(attacked, new Player(0, 2, 1, 1));
        assert.deepEqual(attacker, new Player(1, 0, 1, 1));
    });

    it("Should simulate a succeeding combat round", () => {
        let attacked = new Player(1, 3, 1, 1);
        let attacker = new Player(1, 5, 1, 1);
        attack(attacked, [attacker]);
        assert.deepEqual(attacked, new Player(0, 0, 1, 0));
        assert.deepEqual(attacker, new Player(1, 1, 1, 2));
    });

    it("Should simulate a drawing combat round", () => {
        let attacked = new Player(1, 3, 1, 1);
        let attacker = new Player(1, 4, 1, 1);
        attack(attacked, [attacker]);
        assert.deepEqual(attacked, new Player(0, 0, 1, 1));
        assert.deepEqual(attacker, new Player(1, 0, 1, 1));
    });

    it("Should simulate two factions fighting for a siege", () => {
        for (let perm of [false, true]) {
            let attacked = new Player(1, 3, 1, 1);
            let attacker_1 = new Player(1, 2, 1, 1);
            let attacker_2 = new Player(1, 7, 1, 1);
            attack(attacked, perm ? [attacker_1, attacker_2] : [attacker_2, attacker_1]);
            assert.deepEqual(attacked, new Player(0, 0, 1, 0));
            assert.deepEqual(attacker_1, new Player(1, 0, 1, 1));
            assert.deepEqual(attacker_2, new Player(1, 1, 1, 2));
        }
    });

    it("Should simulate two factions annihilating for a siege", () => {
        for (let perm of [false, true]) {
            let attacked = new Player(1, 3, 1, 1);
            let attacker_1 = new Player(1, 2, 1, 1);
            let attacker_2 = new Player(1, 2, 1, 1);
            attack(attacked, perm ? [attacker_1, attacker_2] : [attacker_2, attacker_1]);
            assert.deepEqual(attacked, new Player(1, 3, 1, 1));
            assert.deepEqual(attacker_1, new Player(1, 0, 1, 1));
            assert.deepEqual(attacker_2, new Player(1, 0, 1, 1));
        }
    });

    it("Should simulate two factions drawing for a siege", () => {
        for (let perm of [false, true]) {
            let attacked = new Player(1, 3, 1, 1);
            let attacker_1 = new Player(1, 2, 1, 1);
            let attacker_2 = new Player(1, 6, 1, 1);
            attack(attacked, perm ? [attacker_1, attacker_2] : [attacker_2, attacker_1]);
            assert.deepEqual(attacked, new Player(0, 0, 1, 1));
            assert.deepEqual(attacker_1, new Player(1, 0, 1, 1));
            assert.deepEqual(attacker_2, new Player(1, 0, 1, 1));
        }
    });

    it("Should simulate three factions fighting for a siege", () => {
        for (let perm of permutations([0, 1, 2])) {
            let attacked = new Player(2, 2, 2, 3);
            let attackers = [
                new Player(3, 20, 3, 2),
                new Player(2, 15, 2, 1),
                new Player(1, 13, 3, 1),
            ];

            attack(attacked, perm.map(i => attackers[i]));
            assert.deepEqual(attacked, new Player(0, 0, 2, 2));
            assert.deepEqual(attackers[0], new Player(3, 1, 3, 3));
            assert.deepEqual(attackers[1], new Player(2, 0, 2, 1));
            assert.deepEqual(attackers[2], new Player(1, 0, 3, 1));
        }
    });
});
