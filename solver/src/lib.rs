use std::array;

use profit_sim as sim;
use sim::{BuildingKind, Resources, Sim, PRODUCT_TYPES};

pub use cluster::*;
pub use network::*;

mod cluster;
mod network;
#[cfg(test)]
mod test;

pub fn filter_products(sim: &Sim) -> [bool; PRODUCT_TYPES] {
    let mut resources = Resources::default();
    for b in sim.buildings.iter() {
        if let BuildingKind::Deposit(deposit) = &b.kind {
            resources.values[deposit.resource_type as usize] += deposit.resources;
        }
    }

    array::from_fn(|i| resources.has_at_least(&sim.products[i].resources))
}
