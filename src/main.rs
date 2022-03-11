// use obelisk::*;
use obelisk::genetic_basic::*;
use scoped_threadpool::Pool;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

pub fn main() -> std::io::Result<()> {
    const N: usize = 100;
    let settings = SimulationSettings {
        sub_rounds: 100,
        group_size: 12,
        n_steps: 50,
        population: 100 * N,
        new_population: 10 * N,
        retain_population: 75 * N,
        reproduce_population: 10 * N,
        mutation: 0.2,
        radiation: 0.01,
        sexuated_reproduction: true,

        ..Default::default()
    };

    println!("Initializing {} agents...", settings.population);
    let mut agents = new_agents(settings);
    println!("Initialization done!");

    const N_THREADS: usize = 16;
    let mut pool = Pool::new(N_THREADS as u32);

    for round in 1..=2000 {
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
            *x = x.sqrt();
        }

        if round % 50 == 0 {
            println!("=== Round {} ===", round);
            print_best(&agents, &loss, 20);

            let mut file = File::create(format!("target/tmp-{}.json", round))?;
            write!(
                file,
                "{}",
                serde_json::to_string(&agents).expect("Couldn't serialize agents!")
            )?;
        } else {
            println!("Round {}", round);
        }

        agents = selection(agents, loss, settings);
    }

    let mut file = File::create("target/out.json")?;
    write!(
        file,
        "{}",
        serde_json::to_string(&agents).expect("Couldn't serialize agents!")
    )?;

    Ok(())
}
