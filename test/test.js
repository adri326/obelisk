import assert from "node:assert";
import {Player, update, clean, attack} from "../index.js";

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

    it("Should simulate an accurate combat round", () => {
        let attacked = new Player(1, 3, 1, 1);
        let attacker = new Player(1, 2, 1, 1);
        attack(attacked, [attacker]);
        assert.deepEqual(attacked, new Player(0, 2, 1, 1));
        assert.deepEqual(attacker, new Player(1, 0, 1, 1));
    });
});
