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
fn find_conveyor_connection() {
    let board = Board::new(20, 20);
    let products = Products::default();
    let mut sim = Sim::new(products, board, 20, 20);

    let building = Building::Factory(Factory::new((4, 2), ProductType::Type0));
    let factory_id = sim::place_building(&mut sim, building).unwrap();

    let building = Building::Conveyor(Conveyor::new((10, 3), Rotation::Left, false));
    let conveyor_id = sim::place_building(&mut sim, building).unwrap();

    let ctx = Context {
        sim: &mut sim,
        tree: ConnectionTree::new(),
        distance_map: DistanceMap::new(20, 20),
        factory_id,
    };

    let path_len = find_connection(&ctx, conveyor_id);
    assert_eq!(path_len, Some(1));
}

#[test]
fn find_conveyor_connection_at() {
    let board = Board::new(20, 20);
    let products = Products::default();
    let mut sim = Sim::new(products, board, 20, 20);

    let building = Building::Factory(Factory::new((4, 2), ProductType::Type0));
    let factory_id = sim::place_building(&mut sim, building).unwrap();

    let building = Building::Conveyor(Conveyor::new((10, 3), Rotation::Left, false));
    let conveyor_id = sim::place_building(&mut sim, building).unwrap();

    let ctx = Context {
        sim: &mut sim,
        tree: ConnectionTree::new(),
        distance_map: DistanceMap::new(20, 20),
        factory_id,
    };

    let search_depth = 2;
    let parent_id = NodeId(324);
    let (state, stats) =
        find_connection_at(&ctx, parent_id, Id(32423), Pos::new(11, 3), search_depth).unwrap();
    let path_len = 1;
    let dist = path_len_dist_score(path_len);
    assert_eq!(state, State::Merged(path_len));
    assert_eq!(stats, Some((parent_id, PathStats::new(dist, search_depth))));
}
