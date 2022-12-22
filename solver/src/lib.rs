use std::collections::HashMap;

use profit_sim as sim;
use sim::{
    Building, Id, Pos, ResourceType, Resources, Sim, FACTORY_SIZE, PRODUCT_TYPES, RESOURCE_TYPES,
};

pub use distance::*;
pub use region::*;

mod distance;
mod region;
#[cfg(test)]
mod test;

struct DepositStats {
    id: Id,
    resource_type: ResourceType,
    weight: f32,
}

struct CellStats {
    pos: Pos,
    min: WeightedDist,
    avg: WeightedDist,
    max: WeightedDist,

    score: WeightedDist,
}

#[derive(Debug)]
struct WeightedDist {
    dist: f32,
    weighted: f32,
}

// TODO: consider using a `bumpalo` to limit allocations
pub fn factory_positions(sim: &Sim) {
    let regions = find_regions(sim);
    let possible_products = possible_products_per_region(sim, &regions);
    let deposit_distance_maps = sim
        .buildings
        .iter()
        .filter_map(|(i, b)| {
            let Building::Deposit(deposit) = b else { return None };
            let map = map_distances(sim, deposit.pos, deposit.width, deposit.height);
            dbg!(&map);
            Some((i, map))
        })
        .collect::<HashMap<Id, DistanceMap>>();

    for (region, possible_products) in regions.iter().zip(possible_products.iter()) {
        let product_factory_positions = possible_products
            .iter()
            .enumerate()
            .filter_map(|(i, product)| {
                if !product {
                    return None;
                }
                let product = sim.products[i].clone();

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
                        let weight = needed_resources as f32 * deposit.resources as f32;

                        Some(DepositStats { id, resource_type, weight })
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
                        let mut resources_in_reach = [false; RESOURCE_TYPES];
                        for d in deposit_stats.iter() {
                            // find the distance from the outer border
                            let mut dist = u16::MAX;
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (i, -1);
                                if let Some(Some(d)) = deposit_distance_maps[&d.id].get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (i, FACTORY_SIZE);
                                if let Some(Some(d)) = deposit_distance_maps[&d.id].get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (-1, i);
                                if let Some(Some(d)) = deposit_distance_maps[&d.id].get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (FACTORY_SIZE, i);
                                if let Some(Some(d)) = deposit_distance_maps[&d.id].get(pos) {
                                    dist = dist.min(d);
                                }
                            }

                            // Filter out factory positions that can't reach all necessary resources
                            // in time
                            if dist as u32 / 4 < sim.turns {
                                resources_in_reach[d.resource_type as usize] = true;
                            }

                            let dist = dist as f32;
                            let weighted = 1.0 / (dist + 1.0).ln() * d.weight;

                            min.dist = min.dist.min(dist);
                            min.weighted = min.weighted.min(weighted);
                            max.dist = max.dist.max(dist);
                            max.weighted = max.dist.max(weighted);
                            sum.dist += dist;
                            sum.weighted += weighted;
                        }

                        for (num, reachable) in product.resources.iter().zip(resources_in_reach.iter()) {
                            if num > 0 && !reachable {
                                return None;
                            }
                        }

                        let len = deposit_stats.len() as f32;
                        let avg = WeightedDist { dist: sum.dist / len, weighted: sum.weighted / len };

                        // TODO: calculate some meaningful score
                        let score = WeightedDist {
                            dist: 1.0 / (avg.dist + 1.0).ln() * (max.dist + 1.0).ln(),
                            weighted: avg.weighted * (max.weighted + 1.0).ln(),
                        };
                        Some(CellStats { pos, min, avg, max, score })
                    })
                    .collect::<Vec<_>>();

                // normalize values
                let mut min_score = WeightedDist { dist: f32::MAX, weighted: f32::MAX } ;
                let mut max_score = WeightedDist { dist: 0.0, weighted: 0.0 } ;
                for d in factory_positions.iter() {
                    min_score.dist = min_score.dist.min(d.score.dist);
                    min_score.weighted = min_score.weighted.min(d.score.weighted);
                    max_score.dist = max_score.dist.max(d.score.dist);
                    max_score.weighted = max_score.weighted.max(d.score.weighted);
                }
                factory_positions.iter_mut().for_each(|d| {
                    d.score.dist = (d.score.dist - min_score.dist) / (max_score.dist - min_score.dist);
                    d.score.weighted = (d.score.weighted - min_score.weighted) / (max_score.weighted - min_score.weighted);
                });

                // rank by score
                factory_positions .sort_by(|f1, f2| {
                    let score1 = f1.score.dist + f1.score.weighted;
                    let score2 = f2.score.dist + f2.score.weighted;
                    score2.total_cmp(&score1)
                });

                Some((product, factory_positions))
            })
            .collect::<Vec<_>>();

        println!("------------------------------");
        println!("{:?}", region.deposits);
        println!("{possible_products:?}");
        println!("------------------------------");

        // TODO
        for (product, factory_positions) in product_factory_positions {
            println!("{product:?}");
            println!("------------------------------");
            for f in factory_positions.iter().take(100) {
                println!("{}: {:10}, {:10}", f.pos, f.score.dist, f.score.weighted);
            }
        }
    }
}

pub fn possible_products_per_region(sim: &Sim, regions: &Regions) -> Vec<[bool; PRODUCT_TYPES]> {
    // TODO: consider filtering out products that are technically in the region but aren't reachable
    // by mines/conveyors
    regions
        .iter()
        .map(|r| {
            let mut possible_products = [false; PRODUCT_TYPES];
            let mut available_resources = Resources::default();

            for id in r.deposits.iter() {
                let Building::Deposit(deposit) = &sim.buildings[*id] else { continue };
                available_resources.values[deposit.resource_type as usize] += deposit.resources;
            }

            for (i, p) in sim.products.iter().enumerate() {
                if p.points == 0 {
                    continue;
                }

                if available_resources.has_at_least(&p.resources) {
                    possible_products[i] = true;
                }
            }

            possible_products
        })
        .collect()
}
