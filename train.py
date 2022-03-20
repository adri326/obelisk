import math
import csv
import json
import time
import random
# The following imports need to be installed with pip:
import numpy
from bidict import bidict
import tensorflow as tf
from tensorflow.keras import layers

def read_bidict(path, ignore_first_line=True, parse_nums=True):
    res = bidict()
    line = 0
    for row in csv.reader(open(path), delimiter=','):
        if line == 0 and ignore_first_line:
            line += 1
            continue
        left = row[0]
        right = row[1]

        if parse_nums:
            try:
                left = int(row[0])
            except Exception:
                pass

            try:
                right = int(row[1])
            except Exception:
                pass

        res.put(left, right)
        line += 1
    return res

actions_map = read_bidict("./actions.csv")

def parse_action(action):
    global actions_map
    if type(action) == dict:
        key = [x for x in action.keys()][0]
        return actions_map[key] + action[key]
    else:
        return actions_map[action]

def read_training(path):
    raw = json.load(open(path))
    res = []
    for row in raw:
        previous_actions = []
        players = []
        best_actions = []
        for n in range(len(row["players"])):
            previous_actions.append([])

        for step in row["previous_actions"]:
            for n, action in enumerate(step):
                previous_actions[n].append(parse_action(action))

        for player in row["players"]:
            players.append([
                player["walls"],
                player["soldiers"],
                player["barracks"],
                player["obelisks"],
                int(player["defense"] > 0),
                int(player["defense"] >= 2)
            ])


        for best_action in row["best_actions"]:
            best_actions.append(parse_action(best_action[0]))

        res.append([previous_actions, players, best_actions])
    return res

print(actions_map)

MAX_PLAYERS = 16
ACTION_ATTACK = parse_action("Attack")
MAX_ACTIONS = ACTION_ATTACK + MAX_PLAYERS - 1
N_ACTIONS = 8
MAX_WALLS = 10
MAX_BARRACKS = 10
MAX_OBELISKS = 10
SOLDIERS_SCALE = 5
PERMUTATIONS = 1
SKIP_RATE = 0.5
# previous moves + players
INPUT_SIZE = MAX_ACTIONS * N_ACTIONS + 6 * MAX_PLAYERS

def categorize(value, max):
    res = []
    for n in range(max):
        res.append(1 if n == value else 0)
    return res

def flatten(l):
    res = []
    for sublist in l:
        for item in sublist:
            res.append(item)
    return res

def refine_training(training):
    res_output = []
    res_input = []
    for [previous_actions, raw_players, best_actions] in training:
        for n, playable in filter(lambda x: best_actions[x[0]] != 0, enumerate(raw_players)):
            if random.random() < SKIP_RATE:
                continue
            prev = []
            for o in range(N_ACTIONS):
                index = len(previous_actions[n]) - 1 - o
                if index < 0:
                    prev.append(0)
                else:
                    action = previous_actions[n][o]
                    # The player can't attack itself, so might as well remove it
                    if action >= ACTION_ATTACK + n:
                        prev.append(action - 1)
                    else:
                        prev.append(action)

            players = [playable]
            for o, other in filter(lambda x: x[0] != n, enumerate(raw_players)):
                players.append(other)

            if best_actions[n] >= ACTION_ATTACK + n:
                best_action = best_actions[n] - 1
            else:
                best_action = best_actions[n]

            # for perm_n in range(PERMUTATIONS):
            perm = [x for x in range(MAX_PLAYERS - 1)]
            # random.shuffle(perm)

            # Let's order them by strength instead
            def sort_strength(index):
                if index + 1 < len(players):
                    player = players[index + 1]
                    return -(player[0] * (2 if player[4] > 0 else 1) + player[1])
                else:
                    return 1000

            perm.sort(key = sort_strength)

            action_map = bidict()
            for a in range(ACTION_ATTACK):
                action_map.put(a, a)
            for p in range(MAX_PLAYERS - 1):
                action_map.put(ACTION_ATTACK + p, ACTION_ATTACK + perm[p])

            transformed_players = []
            for p in range(MAX_PLAYERS):
                if p == 0:
                    index = 0
                else:
                    index = perm[p - 1] + 1

                if index < len(players):
                    player = players[index]
                    transformed_players.append([
                        player[0] / MAX_WALLS,
                        1 - math.exp(-player[1] / SOLDIERS_SCALE),
                        player[2] / MAX_BARRACKS,
                        player[3] / MAX_OBELISKS,
                        player[4],
                        player[5]
                    ])
                else:
                    transformed_players.append([0, 0, 0, 0, 0, 0])

            # best_action = categorize(best_action, MAX_ACTIONS)

            transformed_prev = []
            for action in prev:
                transformed_prev.append(categorize(action_map[action], MAX_ACTIONS))

            row = flatten(transformed_prev) + flatten(transformed_players)
            res_input.append(row)
            res_output.append(action_map[best_action])
            assert len(row) == INPUT_SIZE
    return res_input,res_output

if __name__ == "__main__":
    # Read the training data as a bunch of numbers
    training = read_training("./target/train-last.json")
    print(len(training), "training games")

    # Turn the training data into a list of tensors for the AI
    random.shuffle(training)
    refined_x,refined_y = refine_training(training)
    train_x = numpy.array(refined_x)
    train_y = numpy.array(refined_y)

    print(train_x.shape, train_y.shape)

    try:
        model = tf.keras.models.load_model("target/model.h5")
    except IOError:
        print("Creating a new model!")
        model = tf.keras.Sequential([
            layers.Input(shape=(INPUT_SIZE,)),
            # layers.Dense(units = 180, activation="relu"),
            # layers.Dropout(0.2),
            layers.Dense(units = 96, activation="relu"),
            layers.Dropout(0.25),
            layers.Dense(units = 64, activation="relu"),
            layers.Dropout(0.25),
            layers.Dense(units = 48, activation="relu"),
            layers.Dropout(0.25),
            layers.Dense(units = 24, activation="relu"),
            layers.Dropout(0.25),
            layers.Dense(units = MAX_ACTIONS, activation="softmax")
        ])

        # loss: categorical cross-entropy
        model.compile(
            optimizer="adam",
            loss="sparse_categorical_crossentropy",
            metrics=["accuracy"]
        )

    history = model.fit(
        x=train_x,
        y=train_y,
        validation_split=0.1,
        epochs=40,
        verbose=1
    )

    current_time = time.strftime("%Y_%m_%d-%H_%M_%S")
    model.save(f'target/model-{current_time}.h5')
    model.save("target/model.h5")
