use profit_sim as sim;
use sim::{
    Board, Building, Product, ProductType, Products, ResourceType, Resources, Rotation, Sim, SimRun,
};

fn main() {
    for _ in 0..1000000 {
        let mut products = Products::default();
        products[0] = Product::new(Resources::new([7, 0, 0, 0, 0, 0, 0, 0]), 9);

        let mut sim = Sim::new(products, Board::new(20, 10));

        let building = Building::deposit((0, 0), ResourceType::Type0, 4, 4);
        sim::place_building(&mut sim, building).unwrap();

        let building = Building::mine((5, 1), Rotation::Up);
        sim::place_building(&mut sim, building).unwrap();

        let building = Building::factory((8, 0), ProductType::Type0);
        sim::place_building(&mut sim, building).unwrap();

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
