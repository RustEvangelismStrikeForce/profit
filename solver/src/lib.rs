use profit_sim as sim;
use sim::{pos, BuildingKind, Resources, Sim, MAX_BOARD_SIZE, PRODUCT_TYPES, RESOURCE_TYPES};

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

pub fn solve(sim: &Sim) {
    // Inputs are encoded in the following way:
    // First come the 8 products
    // - 8 inputs for resources needed
    // - 1 input for the points
    // summing up to 72
    //
    // Then each cell of the board by rows:
    // - 8 inputs for the resource type of a deposit
    // - 1 input if the cell is an obstacle
    const INPUT_SIZE: usize = (RESOURCE_TYPES + 1) * PRODUCT_TYPES
        + (RESOURCE_TYPES + 1) * MAX_BOARD_SIZE as usize * MAX_BOARD_SIZE as usize;

    // Outputs are encoded in the following way:
    // Each cell by rows:
    // - 8 outputs for the product type of a factory
    // - 1 output if the cell should be the input of a mine
    // - 1 output if the cell should be a connector (either a conveyor or combiner)
    const OUTPUT_SIZE: usize =
        (PRODUCT_TYPES + 1 + 1) * MAX_BOARD_SIZE as usize * MAX_BOARD_SIZE as usize;
    const HIDDEN_SIZE: usize = MAX_BOARD_SIZE as usize * MAX_BOARD_SIZE as usize;
    const HIDDEN_LAYERS: usize = 12;
    let nn = Network::<INPUT_SIZE, OUTPUT_SIZE, HIDDEN_SIZE, HIDDEN_LAYERS>::random();

    let mut input = [0.0; INPUT_SIZE];
    for (i, p) in sim.products.iter().enumerate() {
        for (j, r) in p.resources.values.iter().enumerate() {
            input[i * 9 + j] = *r as f32;
        }
        input[i * 9 + 8] = p.points as f32;
    }

    const OFFSET: usize = (RESOURCE_TYPES + 1) * PRODUCT_TYPES;
    for y in 0..sim.board.height {
        for x in 0..sim.board.width {
            let pos = pos(x, y);
            if let Some(cell) = sim.board[pos] {
                let building = &sim.buildings[cell.id];
                match &building.kind {
                    BuildingKind::Deposit(deposit) => {
                        input[OFFSET
                            + y as usize * MAX_BOARD_SIZE as usize
                            + x as usize
                            + deposit.resource_type as usize] = 1.0;
                    }
                    BuildingKind::Obstacle(_) => {
                        input[OFFSET
                            + y as usize * MAX_BOARD_SIZE as usize
                            + x as usize
                            + RESOURCE_TYPES] = 1.0;
                    }
                    BuildingKind::Mine(_)
                    | BuildingKind::Conveyor(_)
                    | BuildingKind::Combiner(_)
                    | BuildingKind::Factory(_) => todo!(),
                }
            }
        }
    }

    let output = nn.calc(input);
}
