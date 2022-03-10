use obelisk::*;
use obelisk::genetic_basic::*;
use scoped_threadpool::Pool;
use std::sync::Mutex;

pub fn main() {
    let settings = SimulationSettings {
        sub_rounds: 50,
        group_size: 12,
        n_steps: 40,
        population: 10000,
        new_population: 1000,
        retain_population: 5000,
        reproduce_population: 2000,
        mutation: 0.4,
        sexuated_reproduction: true,

        ..Default::default()
    };

    println!("Initializing {} agents...", settings.population);
    let mut agents = new_agents(settings);
    println!("Initialization done!");

    const N_THREADS: usize = 16;
    let mut pool = Pool::new(N_THREADS as u32);

    for round in 1..=400 {
        let losses = Mutex::new(Vec::with_capacity(N_THREADS));
        pool.scoped(|scope| {
            for _ in 0..N_THREADS {
                let losses = &losses;
                let agents = &agents;
                scope.execute(move || {
                    let loss = simulate_round(agents, settings);
                    losses.lock().unwrap().push(loss);
                });
            }
        });
        let mut loss = vec![0.0; settings.population];
        for l in losses.into_inner().unwrap() {
            for (i, x) in l.into_iter().enumerate() {
                loss[i] += x;
            }
        }

        for x in loss.iter_mut() {
            *x /= N_THREADS as f64;
        }

        if round % 10 == 0 {
            println!("=== Round {} ===", round);
            print_best(&agents, &loss, 10);
        }

        agents = selection(agents, loss, settings);
    }
}
