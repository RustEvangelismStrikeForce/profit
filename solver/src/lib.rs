use profit_sim as sim;
use sim::{BuildingKind, Resources, Sim, PRODUCT_TYPES};

pub use network::*;
pub use region::*;

mod network;
mod region;
#[cfg(test)]
mod test;

pub fn filter_products(sim: &Sim, regions: &Regions) -> [bool; PRODUCT_TYPES] {
    let mut craftable_products = [false; PRODUCT_TYPES];

    for r in regions.iter() {
        let mut resources = Resources::default();

        for &id in r.buildings.iter() {
            let b = &sim.buildings[id];
            if let BuildingKind::Deposit(deposit) = &b.kind {
                resources.values[deposit.resource_type as usize] += deposit.resources;
            }
        }

        for i in 0..PRODUCT_TYPES {
            if resources.has_at_least(&sim.products[i].resources) {
                craftable_products[i] = true;
            }
        }
    }

    craftable_products
}
