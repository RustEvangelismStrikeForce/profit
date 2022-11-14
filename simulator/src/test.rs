use std::array;
use std::fs::File;

use crate::{dto, sim::*};

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

#[test]
fn serialize_example() {
    let file = File::open("../examples/001.task.json").unwrap();
    let serialized: dto::Task = serde_json::from_reader(file).unwrap();
    assert_eq!(
        serialized,
        dto::Task {
            width: 30,
            height: 20,
            objects: vec![
                dto::Object {
                    kind: dto::ObjectKind::Deposit,
                    subtype: 0,
                    x: 1,
                    y: 1,
                    width: Some(5),
                    height: Some(5),
                },
                dto::Object {
                    kind: dto::ObjectKind::Deposit,
                    subtype: 1,
                    x: 1,
                    y: 14,
                    width: Some(5),
                    height: Some(5),
                },
                dto::Object {
                    kind: dto::ObjectKind::Deposit,
                    subtype: 2,
                    x: 22,
                    y: 1,
                    width: Some(7),
                    height: Some(7),
                },
                dto::Object {
                    kind: dto::ObjectKind::Obstacle,
                    subtype: 0,
                    x: 11,
                    y: 9,
                    width: Some(19),
                    height: Some(2),
                },
                dto::Object {
                    kind: dto::ObjectKind::Obstacle,
                    subtype: 0,
                    x: 11,
                    y: 1,
                    width: Some(2),
                    height: Some(8),
                },
            ],
            products: vec![dto::Product {
                subtype: 0,
                resources: [3, 3, 3, 0, 0, 0, 0, 0],
                points: 10
            },],
            turns: 50,
            time: 300,
        }
    )
}
