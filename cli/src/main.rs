use std::sync::Mutex;
use std::time::{Duration, Instant};

use sim::{dto, Sim};

fn main() {
    let start = Instant::now();

    let stdin = std::io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).expect("some input data");

    let task: dto::Task = serde_json::from_str(&input).expect("valid task input format");
    let sim = Sim::try_from(&task).expect("task input to be valid");
    println!("{sim:?}");

    let best_solution = Mutex::new(None);

    std::thread::scope(|s| {
        let (connect_handle, combine_handle) = solver::solve(&sim, s, &best_solution, start);

        let safety_solution_submit_duration = Duration::from_secs_f32(0.2);
        let sim_time_limit = Duration::from_secs_f32(sim.time) - safety_solution_submit_duration;

        loop {
            let now = Instant::now();
            let sleep_duration = sim_time_limit - (now - start);
            if sleep_duration > Duration::from_secs(2) {
                std::thread::sleep(Duration::from_secs(1));
                if connect_handle.is_finished() && combine_handle.is_finished() {
                    break;
                }
            } else {
                std::thread::sleep(sleep_duration);
                break;
            }
        }

        let lock = best_solution.lock().expect("lock not to be poisoned");
        match lock.as_ref() {
            Some(solution) => {
                println!("========================================");
                println!("{:?}\n{:?}", solution.sim.board, solution.run);
                println!("----------------------------------------");
                let dto_solution = dto::Solution::from(&solution.sim);
                let mut stdout = std::io::stdout();
                serde_json::to_writer(&mut stdout, &dto_solution)
                    .expect("at this point we're fucked");
                println!("----------------------------------------");
            }
            None => println!("No solution"),
        };
        // let go of the lock so that the combiner doesn't run into a dead lock
        drop(lock);

        connect_handle.join().unwrap();
        combine_handle.join().unwrap();
    });
}
