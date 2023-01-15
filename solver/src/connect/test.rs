use sim::{Board, ProductType, Products};

use super::*;

#[test]
fn path_stats() {
    assert!(PathStats::new(1, 0) > PathStats::new(4, 0));
    assert!(PathStats::new(1, 0) == PathStats::new(1, 0));
    assert!(PathStats::new(2, 0) < PathStats::new(1, 0));

    assert!(PathStats::new(1, 3) > PathStats::new(1, 1));
    assert!(PathStats::new(1, 2) == PathStats::new(1, 2));
    assert!(PathStats::new(1, 1) < PathStats::new(1, 3));
}

#[test]
fn find_conveyor_connection_around() {
    let board = Board::new(20, 20);
    let products = Products::default();
    let mut sim = Sim::new(products, board, 20, 20.0);

    let factory = Factory::new((4, 2), ProductType::Type0);
    let factory_id = sim::place_building(&mut sim, Building::Factory(factory)).unwrap();

    let distance_map = map_distances(&sim, factory.pos, FACTORY_SIZE, FACTORY_SIZE);
    let mut tree = ConnectionTree::new();
    let ctx = Context {
        sim: &mut sim,
        tree: &mut tree,
        distance_map,
        factory_id,
    };

    let building = Building::Conveyor(Conveyor::new((10, 3), Rotation::Left, false));
    sim::place_building(ctx.sim, building).unwrap();

    let building = Building::Conveyor(Conveyor::new((13, 3), Rotation::Left, false));
    let conveyor_id = sim::place_building(ctx.sim, building).unwrap();

    let search_depth = 2;
    let parent_id = NodeId(324);
    let (state, stats) =
        find_connection_around(&ctx, parent_id, conveyor_id, Pos::new(12, 3), search_depth)
            .unwrap();
    assert_eq!(state, State::Merged);
    assert_eq!(
        stats,
        Some((parent_id, PathStats::new(0, search_depth - 1)))
    );
}
