use profit_sim as sim;
use sim::{Building, Id, Pos, ProductType, ResourceType, Resources, Sim, FACTORY_SIZE};

use connect::*;
pub use distance::*;
pub use region::*;

mod connect;
mod distance;
mod region;
#[cfg(test)]
mod test;

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
    weighted: f32,
    max_products: f32,
}

// TODO: consider using a `bumpalo` to limit allocations
pub fn solve(sim: &Sim) -> sim::Result<()> {
    let regions = find_regions(sim);
    let deposit_distance_maps = map_deposit_distances(sim);

    for region in regions.iter() {
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

                        let mut max = WeightedDist { dist: 0.0, weighted: 0.0 };
                        let mut sum = WeightedDist { dist: 0.0, weighted: 0.0 };
                        let mut resources_in_reach = available_resources;
                        let mut deposits_in_reach = Vec::with_capacity(region.deposits.len());
                        for (idx, ds) in deposit_stats.iter().enumerate() {
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
                            for i in 1..FACTORY_SIZE - 1 {
                                let pos = pos + (0, i);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 1..FACTORY_SIZE - 1 {
                                let pos = pos + (FACTORY_SIZE - 1, i);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }

                            let deposit_idx = DepositIdx { idx, dist };
                            let dist = dist as f32;
                            let weighted = 1.0 / (dist + 1.0).ln() * ds.weight;

                            max.dist = max.dist.max(dist);
                            max.weighted = max.dist.max(weighted);
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
                            weighted: avg.weighted * (max.weighted + 1.0).ln(),
                            max_products: 1.0 / (max_products + 2.0).ln(),
                        };

                        Some(FactoryStats { pos, score, resources_in_reach, deposits_in_reach })
                    })
                    .collect::<Vec<_>>();

                // normalize score components
                let mut min_score = Score { dist: f32::MAX, weighted: f32::MAX, max_products: f32::MAX };
                let mut max_score = Score { dist: 0.0, weighted: 0.0, max_products: 0.0 };
                for d in factory_stats.iter() {
                    min_score.dist = min_score.dist.min(d.score.dist);
                    min_score.weighted = min_score.weighted.min(d.score.weighted);
                    min_score.max_products = min_score.max_products.min(d.score.max_products);
                    max_score.dist = max_score.dist.max(d.score.dist);
                    max_score.weighted = max_score.weighted.max(d.score.weighted);
                    max_score.max_products = max_score.max_products.max(d.score.max_products);
                }
                // increase the range by an epsilon to avoid `NaN`s when all scores are the same
                const EPSILON: f32 = 0.001;
                max_score.dist += EPSILON;
                max_score.weighted += EPSILON;
                max_score.max_products += EPSILON;
                factory_stats.iter_mut().for_each(|d| {
                    d.score.dist = (d.score.dist - min_score.dist) / (max_score.dist - min_score.dist);
                    d.score.weighted = (d.score.weighted - min_score.weighted) / (max_score.weighted - min_score.weighted);
                    d.score.max_products = (d.score.max_products - min_score.max_products) / (max_score.max_products - min_score.max_products);
                });

                // rank by score
                factory_stats.sort_by(|f1, f2| {
                    let score1 = f1.score.dist + f1.score.weighted + f1.score.max_products;
                    let score2 = f2.score.dist + f2.score.weighted + f2.score.max_products;
                    score2.total_cmp(&score1)
                });

                Some(ProductStats { product_type, deposit_stats, factory_stats })
            }).collect::<Vec<_>>();

        for (id, m) in deposit_distance_maps.iter() {
            println!("{id:?} {m:?}");
        }
        println!("------------------------------");
        println!("{:?}", region.deposits);
        println!("------------------------------");

        // TODO: calculate search depth dynamically based on some heuristic using time and board size
        let search_depth = 2;
        let mut current_sim = sim.clone();
        // TODO: try out some combinations of factories producing different products and rank those
        // combinations
        for product_stats in product_stats.iter() {
            for factory_stats in product_stats.factory_stats.iter().take(30) {
                current_sim.clone_from(sim);

                println!(
                    "{}: {:16}, {:16}, {:16}",
                    factory_stats.pos,
                    factory_stats.score.dist,
                    factory_stats.score.weighted,
                    factory_stats.score.max_products
                );

                let res = connect_deposits_and_factory(
                    &mut current_sim,
                    product_stats,
                    factory_stats,
                    search_depth,
                );
                if let Err(e) = res {
                    println!("{e}");
                }
            }
        }
    }

    Ok(())
}
