use std::sync::{mpsc, Mutex};
use std::thread::{self, ScopedJoinHandle};
use std::time::Instant;

use sim::Sim;

use combine::*;
use connect::*;
pub use distance::*;
pub use error::*;
pub use region::*;
pub use stats::*;

mod combine;
mod connect;
mod distance;
mod error;
mod region;
mod stats;
#[cfg(test)]
mod test;

pub fn solve<'env, 'scope>(
    sim: &'env Sim,
    scope: &'scope thread::Scope<'scope, 'env>,
    best_solution: &'scope Mutex<Option<ScoredSolution>>,
    start: Instant,
) -> (ScopedJoinHandle<'scope, ()>, ScopedJoinHandle<'scope, ()>) {
    let regions = find_regions(sim);
    let deposit_distance_maps = map_deposit_distances(sim);
    let region_stats = rank_regional_factory_positions(sim, regions, deposit_distance_maps);

    let (sender, receiver) = mpsc::channel();
    let num_regions = region_stats.len();

    let connect_handle = scope.spawn(move || {
        regional_connections(sim, &region_stats, sender, start);
    });
    let combine_handle = scope.spawn(move || {
        combine::combine_solutions(receiver, best_solution, num_regions);
    });

    (combine_handle, connect_handle)
}

fn regional_connections(
    sim: &Sim,
    region_stats: &[RegionStats],
    sender: mpsc::Sender<CombineMessage>,
    start: Instant,
) {
    'outer: for search_depth in 2..=255 {
        let mut product_iter_indices = vec![0; region_stats.len()];

        let mut region_iters = region_stats
            .iter()
            .map(|r| {
                r.product_stats
                    .iter()
                    .map(|p| (p, p.factory_stats.iter()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut tree = ConnectionTree::new();
        loop {
            // TODO: try out some combinations of factories producing different products and rank those
            // combinations
            let mut all_done = true;
            for (region_idx, region_iter) in region_iters.iter_mut().enumerate() {
                let Some((product_stats, factory_stats_iter)) = region_iter.get_mut(product_iter_indices[region_idx]) else { continue };
                all_done = false;

                let Some(factory_stats) = factory_stats_iter.next() else {
                    // TODO: try out different products
                    product_iter_indices[region_idx] += 1;
                    continue;
                };

                let solution = connect_deposits_and_factory(
                    sim,
                    &mut tree,
                    product_stats,
                    factory_stats,
                    search_depth,
                );

                if let Ok(solution) = solution {
                    sender
                        .send(CombineMessage::Some((region_idx, solution)))
                        .expect("a receiver");
                }
            }

            let now = Instant::now();
            if (now - start).as_secs_f32() > sim.time - 0.1 {
                break 'outer;
            }

            if all_done {
                break;
            }
        }
    }

    sender.send(CombineMessage::Done).expect("a receiver");
}
