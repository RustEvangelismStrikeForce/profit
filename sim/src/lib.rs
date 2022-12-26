use std::collections::HashMap;

pub use board::*;
pub use building::*;
pub use error::*;

mod board;
mod building;
pub mod dto;
mod error;
#[cfg(test)]
mod test;

enum ResourceContainer {
    Deposit(ResourceType, u16),
    Connector(Resources),
    Factory(ProductType, Resources),
}

impl ResourceContainer {
    fn output_resources(&mut self) -> Resources {
        match self {
            Self::Deposit(resource_type, resources) => {
                let num = (*resources).min(3);
                *resources -= num;

                let mut res = Resources::default();
                res[*resource_type] += num;
                res
            }
            Self::Connector(resources) => std::mem::take(resources),
            Self::Factory(_, _) => unreachable!("Facotories cannot output resources"),
        }
    }

    fn input_resources(&mut self, res: Resources) {
        match self {
            Self::Deposit(_, _) => unreachable!("Deposits cannot input resources"),
            Self::Connector(resources) => *resources += res,
            Self::Factory(_, resources) => *resources += res,
        }
    }
}

struct ResourceConnection {
    output_id: Id,
    resources: Resources,
    input_id: Id,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimRun {
    pub rounds: u32,
    pub points: u32,
    pub at_turn: u32,
}

pub fn run(sim: &Sim) -> SimRun {
    let mut points = 0;
    let mut turn = 0;
    let mut at_turn = 0;

    let mut containers = sim
        .buildings
        .iter()
        .filter_map(|(id, b)| {
            let container = match b {
                Building::Deposit(d) => ResourceContainer::Deposit(d.resource_type, d.resources()),
                Building::Mine(_) | Building::Conveyor(_) | Building::Combiner(_) => {
                    ResourceContainer::Connector(Resources::default())
                }
                Building::Factory(f) => {
                    ResourceContainer::Factory(f.product_type, Resources::default())
                }
                Building::Obstacle(_) => return None,
            };
            Some((id, container))
        })
        .collect::<HashMap<_, _>>();

    let mut connections = sim
        .connections
        .iter()
        .map(|c| ResourceConnection {
            output_id: c.output_id,
            resources: Resources::default(),
            input_id: c.input_id,
        })
        .collect::<Vec<_>>();

    while turn < sim.turns {
        let mut unchanged = true;

        // start of the round
        for con in connections.iter_mut() {
            let building_b = containers
                .get_mut(&con.input_id)
                .expect("There should be a container");
            let res = std::mem::take(&mut con.resources);
            unchanged &= res.is_empty();
            building_b.input_resources(res);
        }

        // end of the round
        for con in connections.iter_mut() {
            let building_a = containers
                .get_mut(&con.output_id)
                .expect("There should be a container");
            con.resources = building_a.output_resources();
            unchanged &= con.resources.is_empty();
        }
        for (_, c) in containers.iter_mut() {
            let ResourceContainer::Factory(product_type, resources) = c else { continue };
            let product = &sim.products[*product_type];
            if resources.has_at_least(&product.resources) {
                let count = (*resources / product.resources)
                    .iter()
                    .min()
                    .unwrap_or_default();

                if count > 0 {
                    *resources -= product.resources * Resources::new([count; 8]);
                    points += product.points * count as u32;
                    at_turn = turn + 1;
                    unchanged = false;
                }
            }
        }

        if unchanged {
            break;
        }

        turn += 1;
    }

    SimRun {
        rounds: turn,
        points,
        at_turn,
    }
}
