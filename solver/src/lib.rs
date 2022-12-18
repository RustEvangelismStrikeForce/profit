use profit_sim as sim;
use sim::{Building, Resources, Sim, PRODUCT_TYPES};

pub use region::*;

mod region;
#[cfg(test)]
mod test;

pub fn factory_positions(sim: &Sim) {
    let regions = find_regions(sim);
    let possible_products = possible_products_per_region(sim, &regions);

    for r in regions.iter() {
        for id in r.deposits.iter() {
            let Building::Deposit(deposit) = sim.buildings[*id] else { continue };
        }
    }
}

pub fn possible_products_per_region(sim: &Sim, regions: &Regions) -> Vec<[bool; PRODUCT_TYPES]> {
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
