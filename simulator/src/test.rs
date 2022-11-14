use std::array;

use crate::sim::*;

#[test]
fn place_mine_rotated_up() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, vec![], vec![], Board::new(10, 10));

    let id = Id(-1);
    let building = Building::new(
        3,
        3,
        BuildingKind::Mine(Mine::new(Rotation::Up, ResourcePipe::default())),
    );

    place_building(&mut sim, &building, id).unwrap();

    let mut expected = Board::new(10, 10);
    expected[building.pos] = Some(Cell::inert(id));
    expected[building.pos + pos(1, 0)] = Some(Cell::inert(id));
    expected[building.pos + pos(0, 1)] = Some(Cell::inert(id));
    expected[building.pos + pos(1, 1)] = Some(Cell::inert(id));
    expected[building.pos + pos(-1, 1)] = Some(Cell::input(id));
    expected[building.pos + pos(2, 1)] = Some(Cell::output(id));
    assert_eq!(sim.board, expected);
}

#[test]
fn place_mine_rotated_right() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, vec![], vec![], Board::new(10, 10));

    let id = Id(-1);
    let building = Building::new(
        3,
        3,
        BuildingKind::Mine(Mine::new(Rotation::Right, ResourcePipe::default())),
    );

    place_building(&mut sim, &building, id).unwrap();

    let mut expected = Board::new(10, 10);
    expected[building.pos + pos(0, 0)] = Some(Cell::inert(id));
    expected[building.pos + pos(1, 0)] = Some(Cell::inert(id));
    expected[building.pos + pos(0, 1)] = Some(Cell::inert(id));
    expected[building.pos + pos(1, 1)] = Some(Cell::inert(id));
    expected[building.pos + pos(0, -1)] = Some(Cell::input(id));
    expected[building.pos + pos(0, 2)] = Some(Cell::output(id));
    assert_eq!(sim.board, expected);
}

#[test]
fn place_mine_rotated_down() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, vec![], vec![], Board::new(10, 10));

    let id = Id(-1);
    let building = Building::new(
        3,
        3,
        BuildingKind::Mine(Mine::new(Rotation::Down, ResourcePipe::default())),
    );

    place_building(&mut sim, &building, id).unwrap();

    let mut expected = Board::new(10, 10);
    expected[building.pos + pos(0, 0)] = Some(Cell::inert(id));
    expected[building.pos + pos(1, 0)] = Some(Cell::inert(id));
    expected[building.pos + pos(0, 1)] = Some(Cell::inert(id));
    expected[building.pos + pos(1, 1)] = Some(Cell::inert(id));
    expected[building.pos + pos(2, 0)] = Some(Cell::input(id));
    expected[building.pos + pos(-1, 0)] = Some(Cell::output(id));
    assert_eq!(sim.board, expected);
}

#[test]
fn place_mine_rotated_left() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, vec![], vec![], Board::new(10, 10));

    let id = Id(-1);
    let building = Building::new(
        3,
        3,
        BuildingKind::Mine(Mine::new(Rotation::Left, ResourcePipe::default())),
    );

    place_building(&mut sim, &building, id).unwrap();

    let mut expected = Board::new(10, 10);
    expected[building.pos + pos(0, 0)] = Some(Cell::inert(id));
    expected[building.pos + pos(1, 0)] = Some(Cell::inert(id));
    expected[building.pos + pos(0, 1)] = Some(Cell::inert(id));
    expected[building.pos + pos(1, 1)] = Some(Cell::inert(id));
    expected[building.pos + pos(1, 2)] = Some(Cell::input(id));
    expected[building.pos + pos(1, -1)] = Some(Cell::output(id));
    assert_eq!(sim.board, expected);
}
