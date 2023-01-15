use std::sync::{mpsc, Mutex};

use sim::{Building, Buildings, Sim, SimRun};

const MAX_COMBINATIONS: f32 = 1000.0;

pub enum CombineMessage {
    Some((usize, ScoredSolution)),
    Done,
}

#[derive(Clone, Debug)]
pub struct ScoredSolution {
    pub sim: Sim,
    pub run: SimRun,
}

impl ScoredSolution {
    pub fn new(sim: Sim, run: SimRun) -> Self {
        Self { sim, run }
    }
}

impl Eq for ScoredSolution {}

impl PartialEq for ScoredSolution {
    fn eq(&self, other: &Self) -> bool {
        self.run.eq(&other.run)
    }
}

impl Ord for ScoredSolution {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.run.cmp(&other.run)
    }
}

impl PartialOrd for ScoredSolution {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn combine_solutions(
    receiver: mpsc::Receiver<CombineMessage>,
    best_solution: &Mutex<Option<ScoredSolution>>,
    num_regions: usize,
) {
    let num_components = (MAX_COMBINATIONS.log(num_regions as f32) as usize).max(1);

    let mut regional_solutions: Vec<Vec<ScoredSolution>> = vec![Vec::new(); num_regions];
    let mut best_local_solution = None;

    while let Ok(message) = receiver.recv() {
        let (region_idx, region_solution) = match message {
            CombineMessage::Some(s) => s,
            CombineMessage::Done => break,
        };

        if num_regions > 1 {
            let mut current_sim = region_solution.sim.clone();

            recursive_permutations(
                &mut current_sim,
                &mut best_local_solution,
                &regional_solutions,
                region_idx,
                0,
                num_components,
            );
        } else {
            let run = sim::run(&region_solution.sim);
            cmp_and_set(&mut best_local_solution, &region_solution.sim, run);
        }

        // update best solution to turn in
        let mut lock = best_solution.lock().expect("lock not to be poisoned");
        *lock = best_local_solution.clone();

        let current_region_solutions = &mut regional_solutions[region_idx];
        #[allow(irrefutable_let_patterns)]
        if let Ok(pos) | Err(pos) = current_region_solutions.binary_search(&region_solution) {
            regional_solutions[region_idx].insert(pos, region_solution);
        }
    }
}

fn recursive_permutations(
    sim: &Sim,
    best_solution: &mut Option<ScoredSolution>,
    region_solutions: &[Vec<ScoredSolution>],
    skip_idx: usize,
    mut region_idx: usize,
    num_components: usize,
) {
    if region_idx == skip_idx {
        region_idx += 1;
    }

    if region_idx >= region_solutions.len() {
        let run = sim::run(sim);
        cmp_and_set(best_solution, sim, run);
        return;
    }

    let solutions = &region_solutions[region_idx];

    let mut current_sim = sim.clone();
    for s in solutions.iter().rev().take(num_components) {
        current_sim.clone_from(sim);
        let res = add_solution_buildings(&mut current_sim, &s.sim.buildings);
        if res.is_err() {
            continue;
        }
        recursive_permutations(
            &current_sim,
            best_solution,
            region_solutions,
            skip_idx,
            region_idx + 1,
            num_components,
        );
    }
}

fn add_solution_buildings(sim: &mut Sim, buildings: &Buildings) -> sim::Result<()> {
    for (_, b) in buildings.iter() {
        match b {
            Building::Deposit(_) | Building::Obstacle(_) => (),
            Building::Mine(_)
            | Building::Conveyor(_)
            | Building::Combiner(_)
            | Building::Factory(_) => {
                sim::place_building(sim, b.clone())?;
            }
        }
    }

    Ok(())
}

fn cmp_and_set(best_solution: &mut Option<ScoredSolution>, sim: &Sim, run: SimRun) {
    match best_solution {
        None => *best_solution = Some(ScoredSolution::new(sim.clone(), run)),
        Some(best) => {
            if run > best.run {
                *best_solution = Some(ScoredSolution::new(sim.clone(), run));
            }
        }
    }
}
