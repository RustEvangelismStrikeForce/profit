use std::array;

use profit_sim as sim;
use sim::{
    pos, Board, Building, BuildingKind, Deposit, Factory, Mine, Product, ProductType, ResourceType,
    Resources, Rotation, Sim, SimRun,
};

fn main() {
    for _ in 0..1000000 {
        let mut products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
        products[0] = Product::new(Resources::new([7, 0, 0, 0, 0, 0, 0, 0]), 9);

        let mut sim = Sim::new(products, Board::new(20, 10));

        sim::add_building(
            &mut sim,
            Building::new(
                pos(0, 0),
                BuildingKind::Deposit(Deposit::new(ResourceType::Type0, 4, 4)),
            ),
        )
        .unwrap();

        sim::add_building(
            &mut sim,
            Building::new(pos(5, 1), BuildingKind::Mine(Mine::new(Rotation::Up))),
        )
        .unwrap();

        sim::add_building(
            &mut sim,
            Building::new(
                pos(8, 0),
                BuildingKind::Factory(Factory::new(ProductType::Type0)),
            ),
        )
        .unwrap();

        let run = sim::run(&mut sim, 100);
        assert_eq!(
            run,
            SimRun {
                rounds: 29,
                points: 99,
                at_turn: 28
            }
        );
    }
}
