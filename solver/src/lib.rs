use profit_sim as sim;
use sim::{BuildingKind, Resources, Sim};

pub use region::*;

mod region;
#[cfg(test)]
mod test;

pub fn possible_products_per_region(sim: &Sim, regions: &Regions) -> Vec<u8> {
    regions
        .iter()
        .map(|r| {
            let mut possible_products = 0x00;
            let mut available_resources = Resources::default();

            for &id in r.buildings.iter() {
                let b = &sim.buildings[id];
                if let BuildingKind::Deposit(deposit) = &b.kind {
                    available_resources.values[deposit.resource_type as usize] += deposit.resources;
                }
            }

            for (i, p) in sim.products.iter().enumerate() {
                if p.points == 0 {
                    continue;
                }

                if available_resources.has_at_least(&p.resources) {
                    possible_products |= 0x01 << i;
                }
            }

            possible_products
        })
        .collect()
}
