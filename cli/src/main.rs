use std::time::Instant;

use sim::{dto, Sim};

fn main() {
    let start = Instant::now();

    let stdin = std::io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();

    let task: dto::Task = serde_json::from_str(&input).unwrap();
    let sim = Sim::try_from(&task).unwrap();
    println!("{sim:?}");

    match solver::solve(&sim, start) {
        Err(e) => println!("{e}"),
        Ok((sim, _)) => {
            println!("----------------------------------------");
            let solution = dto::Solution::from(&sim);
            let mut stdout = std::io::stdout();
            serde_json::to_writer(&mut stdout, &solution).expect("at this point we're fucked");
        }
    };
}
