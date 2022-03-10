use obelisk::*;
use obelisk::genetic_basic::*;

pub fn main() {
    let settings = SimulationSettings {
        sub_rounds: 200,
        group_size: 10,
        n_steps: 50,
        population: 10000,
        retain_population: 5000,
        reproduce_population: 2500,
        mutation: 0.05,
        sexuated_reproduction: true,

        ..Default::default()
    };

    println!("Initializing {} agents...", settings.population);
    let mut agents = new_agents(settings);
    println!("Initialization done!");

    for round in 1..=200 {
        let loss = simulate_round(&agents, settings);

        if round % 20 == 0 {
            println!("=== Round {} ===", round);
            print_best(&agents, &loss, 10);
        }

        agents = selection(agents, loss, settings);
    }
}
