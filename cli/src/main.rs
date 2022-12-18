use std::io::stdin;

use profit_sim as sim;
use profit_solver as solver;
use sim::{dto, Sim};

fn main() {
    let stdin = stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();

    let task: dto::Task = serde_json::from_str(&input).unwrap();
    let sim = Sim::try_from(&task).unwrap();
    println!("{sim:?}");

    solver::factory_positions(&sim);
}
