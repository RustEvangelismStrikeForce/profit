use std::collections::HashMap;

use profit_sim as sim;
use sim::{Building, Factory, Id, Pos, ProductType, ResourceType, Resources, Sim, FACTORY_SIZE};

pub use distance::*;
pub use region::*;

mod distance;
mod region;
#[cfg(test)]
mod test;

struct DepositStats {
    id: Id,
    resource_type: ResourceType,
    resources: u16,
    weight: f32,
}

struct CellStats {
    pos: Pos,
    score: Score,
    resources_in_reach: Resources,
    /// indices into deposit_stats
    deposits_in_reach: Vec<usize>,
}

#[derive(Debug)]
struct WeightedDist {
    dist: f32,
    weighted: f32,
}

#[derive(Debug)]
struct Score {
    dist: f32,
    weighted: f32,
    max_products: f32,
}

// TODO: consider using a `bumpalo` to limit allocations
pub fn solve(sim: &Sim) -> sim::Result<()> {
    let regions = find_regions(sim);
    let deposit_distance_maps = sim
        .buildings
        .iter()
        .filter_map(|(i, b)| {
            let Building::Deposit(deposit) = b else { return None };
            let map = map_distances(sim, deposit.pos, deposit.width, deposit.height);
            println!("{map:?}");
            Some((i, map))
        })
        .collect::<HashMap<Id, DistanceMap>>();

    for region in regions.iter() {
        let mut available_resources = Resources::default();
        for id in region.deposits.iter() {
            let Building::Deposit(deposit) = &sim.buildings[*id] else { continue };
            available_resources.values[deposit.resource_type as usize] += deposit.resources;
        }

        let product_factory_stats = sim.products.iter()
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
                        let resources = deposit.resources;
                        let weight = needed_resources as f32 * resources as f32;

                        Some(DepositStats { id, resource_type, resources, weight })
                    })
                    .collect::<Vec<_>>();

                let mut factory_positions = region
                    .cells
                    .iter()
                    .filter_map(|&pos| {
                        // check if a factory could even be placed here
                        for y in 0..FACTORY_SIZE {
                            for x in 0..FACTORY_SIZE {
                                let p = pos + (x, y);
                                // out of bounds
                                let cell = sim.board.get(p)?;
                                // cell is non-empty
                                if cell.is_some() {
                                    return None;
                                }
                            }
                        }

                        let mut min = WeightedDist { dist: f32::MAX, weighted: f32::MAX };
                        let mut max = WeightedDist { dist: 0.0, weighted: 0.0 };
                        let mut sum = WeightedDist { dist: 0.0, weighted: 0.0 };
                        let mut resources_in_reach = available_resources;
                        let mut deposits_in_reach = Vec::with_capacity(region.deposits.len());
                        for (di, ds) in deposit_stats.iter().enumerate() {
                            let map = &deposit_distance_maps[&ds.id];
                            // find the distance from the outer border of the factory
                            let mut dist = u16::MAX;
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (i, 0);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (i, FACTORY_SIZE - 1);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (0, i);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (FACTORY_SIZE - 1, i);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }

                            let dist = dist as f32;
                            let weighted = 1.0 / (dist + 1.0).ln() * ds.weight;

                            min.dist = min.dist.min(dist);
                            min.weighted = min.weighted.min(weighted);
                            max.dist = max.dist.max(dist);
                            max.weighted = max.dist.max(weighted);
                            sum.dist += dist;
                            sum.weighted += weighted;

                            if dist == 0.0 {
                                return None;
                            } else if dist as u32 / 4 < sim.turns {
                                deposits_in_reach.push(di);
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
                            weighted: avg.weighted * (max.weighted + 1.0).ln(),
                            max_products: 1.0 / (max_products + 1.0).ln(),
                        };

                        Some(CellStats { pos, score, resources_in_reach, deposits_in_reach })
                    })
                    .collect::<Vec<_>>();

                // normalize scores
                let mut min_score = Score { dist: f32::MAX, weighted: f32::MAX, max_products: f32::MAX };
                let mut max_score = Score { dist: 0.0, weighted: 0.0, max_products: 0.0 };
                for d in factory_positions.iter() {
                    min_score.dist = min_score.dist.min(d.score.dist);
                    min_score.weighted = min_score.weighted.min(d.score.weighted);
                    min_score.max_products = min_score.max_products.min(d.score.max_products);
                    max_score.dist = max_score.dist.max(d.score.dist);
                    max_score.weighted = max_score.weighted.max(d.score.weighted);
                    max_score.max_products = max_score.max_products.max(d.score.max_products);
                }
                factory_positions.iter_mut().for_each(|d| {
                    d.score.dist = (d.score.dist - min_score.dist) / (max_score.dist - min_score.dist);
                    d.score.weighted = (d.score.weighted - min_score.weighted) / (max_score.weighted - min_score.weighted);
                    d.score.max_products = (d.score.max_products - min_score.max_products) / (max_score.max_products - min_score.max_products);
                });

                // rank by score
                factory_positions.sort_by(|f1, f2| {
                    let score1 = f1.score.dist + f1.score.weighted + f1.score.max_products;
                    let score2 = f2.score.dist + f2.score.weighted + f2.score.max_products;
                    score2.total_cmp(&score1)
                });

                let product_type = ProductType::try_from(i as u8).unwrap();
                Some((product_type, product, deposit_stats, factory_positions))
            }).collect::<Vec<_>>();

        println!("------------------------------");
        println!("{:?}", region.deposits);
        println!("------------------------------");

        let mut current_sim = sim.clone();
        for (product_type, product, deposit_stats, factory_positions) in product_factory_stats {
            println!("{product:?}");
            println!("------------------------------");
            for factory_pos in factory_positions.iter() {
                sim.clone_into(&mut current_sim);
                println!(
                    "{}: {:10}, {:10}",
                    factory_pos.pos, factory_pos.score.dist, factory_pos.score.weighted
                );
                let factory = Building::Factory(Factory::new(factory_pos.pos, product_type));
                let res = sim::place_building(&mut current_sim, factory);
                if let Err(e) = res {
                    println!("{e}");
                }

                for di in factory_pos.deposits_in_reach.iter().copied() {
                    let deposit = &deposit_stats[di];

                    // TODO: try to connect all the things
                }
            }
        }
    }

    Ok(())
}
