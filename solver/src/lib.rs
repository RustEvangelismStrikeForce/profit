use std::collections::HashMap;

use profit_sim as sim;
use sim::{Building, Id, Resources, Sim, PRODUCT_TYPES};

pub use distance::*;
pub use region::*;

mod distance;
mod region;
#[cfg(test)]
mod test;

// TODO: consider using a `bumpalo` to limit allocations
pub fn factory_positions(sim: &Sim) {
    let regions = find_regions(sim);
    let possible_products = possible_products_per_region(sim, &regions);
    let distance_maps = sim
        .buildings
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            let Building::Deposit(deposit) = b else { return None };
            Some((
                Id(i as u16),
                map_distances(sim, deposit.pos, deposit.width, deposit.height),
            ))
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

                // Calculate inverse weights of deposits dependent on the needed and available resources
                let deposit_weights = region
                    .deposits
                    .iter()
                    .filter_map(|&id| {
                        let Building::Deposit(deposit) = &sim.buildings[id] else { return None };

                        let needed_resources = product.resources[deposit.resource_type];
                        if needed_resources == 0 {
                            return None;
                        }
                        // TODO: possibly factor in if there are other deposits of the same resource
                        // type in the region
                        let weight = needed_resources as f32 * deposit.resources as f32;

                        Some((id, weight))
                    })
                    .collect::<Vec<_>>();

                let mut factory_positions = region
                    .cells
                    .iter()
                    .map(|&cell| {
                        let distances = deposit_weights
                            .iter()
                            .map(|(id, _)| distance_maps[id][cell].unwrap()); // The cells in the region should be mapped out, since the deposit is either directly inside it or adjacent

                        let sum: f32 = deposit_weights
                            .iter()
                            .zip(distances.clone())
                            .map(|(_, d)| d as f32)
                            .sum();

                        let rank: f32 = deposit_weights
                            .iter()
                            .zip(distances)
                            .map(|((_, w), d)| d as f32 * 1.0 / w)
                            .sum();
                        (cell, 500.0 * rank, sum)
                    })
                    .collect::<Vec<_>>();

                factory_positions
                    .sort_by(|(_, r1, s1), (_, r2, s2)| (r1 + s1).total_cmp(&(r2 + s2)));

                Some((product, factory_positions))
            })
            .collect::<Vec<_>>();

        println!("------------------------------");
        println!("{:?}", region.deposits);
        println!("{possible_products:?}");
        println!("------------------------------");

        // TODO
        for (product, positions) in product_factory_positions {
            println!("{product:?}");
            println!("------------------------------");
            for (p, r, s) in positions.iter().take(100) {
                println!("{p}: {r:10}, {s:10}");
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
