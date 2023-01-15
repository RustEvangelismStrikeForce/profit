use std::collections::HashMap;
use std::sync::{mpsc, Mutex};
use std::thread::{self, ScopedJoinHandle};
use std::time::Instant;

use sim::{Id, Pos, ProductType, ResourceType, Resources, Sim, Building, FACTORY_SIZE};

use combine::*;
use connect::*;
pub use distance::*;
pub use error::*;
pub use region::*;

mod combine;
mod connect;
mod distance;
mod error;
mod region;
#[cfg(test)]
mod test;

struct RegionStats {
    product_stats: Vec<ProductStats>,
}

struct ProductStats {
    product_type: ProductType,
    deposit_stats: Vec<DepositStats>,
    factory_stats: Vec<FactoryStats>,
}

struct DepositStats {
    id: Id,
    resource_type: ResourceType,
    resources: u16,
    weight: f32,
}

struct FactoryStats {
    pos: Pos,
    score: Score,
    resources_in_reach: Resources,
    /// indices into deposit_stats
    deposits_in_reach: Vec<DepositIdx>,
}

struct DepositIdx {
    idx: usize,
    dist: u16,
}

#[derive(Debug)]
struct WeightedDist {
    dist: f32,
    weighted: f32,
}

#[derive(Debug)]
struct Score {
    dist: f32,
    middle: f32,
    weighted: f32,
    max_products: f32,
}

pub fn solve<'env, 'scope>(
    sim: &'env Sim,
    scope: &'scope thread::Scope<'scope, 'env>,
    best_solution: &'scope Mutex<Option<ScoredSolution>>,
    start: Instant,
) -> (ScopedJoinHandle<'scope, ()>, ScopedJoinHandle<'scope, ()>) {
    let regions = find_regions(sim);
    let deposit_distance_maps = map_deposit_distances(sim);
    let region_stats = regional_factory_position_stats(sim, regions, deposit_distance_maps);

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

fn regional_factory_position_stats(
    sim: &Sim,
    regions: Regions,
    deposit_distance_maps: HashMap<Id, DistanceMap>,
) -> Vec<RegionStats> {
    regions.iter().filter_map(|region| {
        let mut available_resources = Resources::default();
        for id in region.deposits.iter() {
            let Building::Deposit(deposit) = &sim.buildings[*id] else { continue };
            available_resources.values[deposit.resource_type as usize] += deposit.resources();
        }

        let product_stats = sim.products.iter()
            .enumerate()
            .filter_map(|(i, product)| {
                if product.points == 0 {
                    return None;
                }
                // filter out products that require more or different resources then there are in the
                // region
                if !available_resources.has_at_least(&product.resources) {
                    return None;
                }

                let product_type = ProductType::try_from(i as u8).unwrap();

                // calculate a weight for a deposit and filter out ones that don't provide any
                // resources needed for the current product
                let deposit_stats = region
                    .deposits
                    .iter()
                    .filter_map(|&id| {
                        let Building::Deposit(deposit) = &sim.buildings[id] else { unreachable!("This should be a deposit") };
                        let resource_type = deposit.resource_type;

                        let needed_resources = product.resources[resource_type];
                        if needed_resources == 0 {
                            return None;
                        }

                        // TODO: possibly factor in if there are other deposits of the same resource
                        // type in the region
                        let resources = deposit.resources();
                        let weight = needed_resources as f32 * resources as f32;

                        Some(DepositStats { id, resource_type, resources, weight })
                    })
                    .collect::<Vec<_>>();

                let mut factory_stats = region
                    .cells
                    .iter()
                    .filter_map(|&factory_pos| {
                        // check if a factory could even be placed here
                        for y in 0..FACTORY_SIZE {
                            for x in 0..FACTORY_SIZE {
                                let p = factory_pos + (x, y);
                                // out of bounds
                                let cell = sim.board.get(p)?;
                                // cell is non-empty
                                if cell.is_some() {
                                    return None;
                                }
                            }
                        }

                        let mut max = WeightedDist { dist: 0.0, weighted: 0.0 };
                        let mut min = WeightedDist {dist: f32::MAX, weighted:f32::MAX};
                        let mut sum = WeightedDist { dist: 0.0, weighted: 0.0 };
                        let mut resources_in_reach = available_resources;
                        let mut deposits_in_reach = Vec::with_capacity(region.deposits.len());
                        for (idx, ds) in deposit_stats.iter().enumerate() {
                            let map = &deposit_distance_maps[&ds.id];
                            // find the distance from the outer border of the factory
                            let mut dist = u16::MAX;
                            for i in 0..FACTORY_SIZE {
                                let pos = factory_pos + (i, 0);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 1..FACTORY_SIZE - 1 {
                                let pos = factory_pos + (0, i);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                                let pos = factory_pos + (FACTORY_SIZE - 1, i);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 0..FACTORY_SIZE {
                                let pos = factory_pos + (i, FACTORY_SIZE - 1);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }

                            let deposit_idx = DepositIdx { idx, dist };
                            let dist = dist as f32;
                            let weighted = ds.weight / (dist + 1.0);

                            max.dist = max.dist.max(dist);
                            max.weighted = max.dist.max(weighted);
                            min.dist = min.dist.min(dist);
                            min.weighted = min.dist.min(weighted);
                            sum.dist += dist;
                            sum.weighted += weighted;


                            if dist == 0.0 {
                                return None;
                            } else if (dist as u32 / 4) + 2 < sim.turns {
                                deposits_in_reach.push(deposit_idx);
                            } else {
                                resources_in_reach[ds.resource_type] -= ds.resources;
                            }
                        }

                        // Filter out factory positions that can't reach all necessary resources
                        // in time
                        if !resources_in_reach.has_at_least(&product.resources) {
                            return None;
                        }

                        let len = deposit_stats.len() as f32;
                        let avg = WeightedDist { dist: sum.dist / len, weighted: sum.weighted / len };

                        // TODO: calculate some meaningful score
                        let max_products = (resources_in_reach / product.resources).iter().min().unwrap_or_default() as f32;
                        let score = Score {
                            dist: 1.0 / (avg.dist + 1.0).ln() * (max.dist + 1.0).ln(),
                            middle: 1.0 / ((max.dist - min.dist).abs() + 1000.0).ln(),
                            weighted: avg.weighted * (max.weighted + 1.0).ln(),
                            max_products: 1.0 / (max_products + 2.0).ln(),
                        };

                        Some(FactoryStats { pos: factory_pos, score, resources_in_reach, deposits_in_reach })
                    })
                    .collect::<Vec<_>>();

                // normalize score components
                let mut min_score = Score { dist: f32::MAX, middle: f32::MAX, weighted: f32::MAX, max_products: f32::MAX };
                let mut max_score = Score { dist: 0.0, middle: 0.0, weighted: 0.0, max_products: 0.0 };
                for d in factory_stats.iter() {
                    min_score.dist = min_score.dist.min(d.score.dist);
                    min_score.middle = min_score.middle.min(d.score.middle);
                    min_score.weighted = min_score.weighted.min(d.score.weighted);
                    min_score.max_products = min_score.max_products.min(d.score.max_products);
                    max_score.dist = max_score.dist.max(d.score.dist);
                    max_score.middle = max_score.middle.max(d.score.middle);
                    max_score.weighted = max_score.weighted.max(d.score.weighted);
                    max_score.max_products = max_score.max_products.max(d.score.max_products);
                }
                // increase the range by an epsilon to avoid `NaN`s when all scores are the same
                const EPSILON: f32 = 0.001;
                max_score.dist += EPSILON;
                max_score.middle += EPSILON;
                max_score.weighted += EPSILON;
                max_score.max_products += EPSILON;
                factory_stats.iter_mut().for_each(|d| {
                    d.score.dist = (d.score.dist - min_score.dist) / (max_score.dist - min_score.dist);
                    d.score.middle = (d.score.middle - min_score.middle) / (max_score.middle - min_score.middle);
                    d.score.weighted = (d.score.weighted - min_score.weighted) / (max_score.weighted - min_score.weighted);
                    d.score.max_products = (d.score.max_products - min_score.max_products) / (max_score.max_products - min_score.max_products);
                });

                // rank by score
                factory_stats.sort_by(|f1, f2| {
                    let score1 = f1.score.dist + f1.score.middle + f1.score.weighted + f1.score.max_products;
                    let score2 = f2.score.dist + f2.score.middle + f2.score.weighted + f2.score.max_products;
                    score2.total_cmp(&score1)
                });

                Some(ProductStats { product_type, deposit_stats, factory_stats })
            }).collect::<Vec<_>>();

        (!product_stats.is_empty()).then_some(RegionStats { product_stats })
    })
    .collect()
}

fn regional_connections(
    sim: &Sim,
    region_stats: &[RegionStats],
    sender: mpsc::Sender<CombineMessage>,
    start: Instant,
) {
    // TODO: calculate search depth dynamically based on some heuristic using time and board size
    let search_depth = 2;
    let mut current_sim = sim.clone();
    let mut product_iter_indices = vec![0; region_stats.len()];

    let mut region_iters = region_stats
        .iter()
        .map(|r| {
            r
                .product_stats
                .iter()
                .map(|p| (p, p.factory_stats.iter()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut i = 0;
    let mut tree = ConnectionTree::new();
    loop {
        // TODO: try out some combinations of factories producing different products and rank those
        // combinations
        let mut all_done = true;
        for (region_idx, region_iter) in region_iters.iter_mut().enumerate() {
            let Some((product_stats, factory_stats_iter)) = region_iter.get_mut(product_iter_indices[region_idx]) else { continue };
            let Some(factory_stats) = factory_stats_iter.next() else {
                // TODO: try out different products
                product_iter_indices[region_idx] += 1;
                continue;
            };

            all_done = false;
            
            println!(
                "{i:4} {}: {:16}, {:16}, {:16} {:16}",
                factory_stats.pos,
                factory_stats.score.dist,
                factory_stats.score.middle,
                factory_stats.score.weighted,
                factory_stats.score.max_products
            );
            i += 1;
            
            current_sim.clone_from(sim);

            let solution = connect_deposits_and_factory(
                &mut current_sim,
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
        if (now - start).as_secs_f32() > sim.time {
            break;
        }

        if all_done {
            break;
        }
    }

    sender.send(CombineMessage::Done).expect("a receiver");
}
