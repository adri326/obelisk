use obelisk::generate_training::*;
use obelisk::genetic_basic::*;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Write;

fn main() -> std::io::Result<()> {
    let agents = std::fs::read_to_string("target/out.json")?;
    let agents: Vec<SimpleAgent> = serde_json::from_str(&agents).expect("Couldn't parse target/out.json");

    let settings = TrainingSettings {
        n_data: 48000,
        samples: 2000,
        n_players: 8..16,
        initial_actions: 0..30,
        threads: 12,
        ..Default::default()
    };

    println!("{:#?}", settings);

    let training_data = generate_training_data_simpleagent(
        settings,
        &agents,
        agents.len() / 2
    );

    let mut file = File::create(format!(
        "target/train-{}.json",
        SystemTime::now().duration_since(UNIX_EPOCH).expect("Uh oh").as_millis()
    ))?;

    let serialized = serde_json::to_string(&training_data).expect("Couldn't serialize training_data!");

    write!(file, "{}", serialized)?;

    let mut file = File::create("target/train-last.json")?;
    write!(file, "{}", serialized)
}
