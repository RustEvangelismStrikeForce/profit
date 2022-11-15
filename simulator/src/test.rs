use std::array;
use std::fs::File;

use crate::{
    dto::{self, Object},
    sim::*,
};

#[test]
fn place_mine_rotated_up() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, vec![], Board::new(10, 10));

    let building = Building::new(
        pos(3, 3),
        BuildingKind::Mine(Mine::new(Rotation::Up, ResourcePipe::default())),
    );

    add_building(&mut sim, building).unwrap();

    let id = Id(0);
    let mut expected = Board::new(10, 10);
    expected[pos(3, 3)] = Some(Cell::inert(id));
    expected[pos(3, 4)] = Some(Cell::inert(id));
    expected[pos(4, 3)] = Some(Cell::inert(id));
    expected[pos(4, 4)] = Some(Cell::inert(id));
    expected[pos(2, 4)] = Some(Cell::input(id));
    expected[pos(5, 4)] = Some(Cell::output(id));
    assert_eq!(sim.board, expected);
}

#[test]
fn place_mine_rotated_right() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, vec![], Board::new(10, 10));

    let building = Building::new(
        pos(3, 3),
        BuildingKind::Mine(Mine::new(Rotation::Right, ResourcePipe::default())),
    );

    add_building(&mut sim, building).unwrap();

    let id = Id(0);
    let mut expected = Board::new(10, 10);
    expected[pos(3, 3)] = Some(Cell::inert(id));
    expected[pos(3, 4)] = Some(Cell::inert(id));
    expected[pos(4, 3)] = Some(Cell::inert(id));
    expected[pos(4, 4)] = Some(Cell::inert(id));
    expected[pos(3, 2)] = Some(Cell::input(id));
    expected[pos(3, 5)] = Some(Cell::output(id));
    assert_eq!(sim.board, expected);
}

#[test]
fn place_mine_rotated_down() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, vec![], Board::new(10, 10));

    let building = Building::new(
        pos(3, 3),
        BuildingKind::Mine(Mine::new(Rotation::Down, ResourcePipe::default())),
    );

    add_building(&mut sim, building).unwrap();

    let id = Id(0);
    let mut expected = Board::new(10, 10);
    expected[pos(3, 3)] = Some(Cell::inert(id));
    expected[pos(3, 4)] = Some(Cell::inert(id));
    expected[pos(4, 3)] = Some(Cell::inert(id));
    expected[pos(4, 4)] = Some(Cell::inert(id));
    expected[pos(5, 3)] = Some(Cell::input(id));
    expected[pos(2, 3)] = Some(Cell::output(id));
    assert_eq!(sim.board, expected);
}

#[test]
fn place_mine_rotated_left() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, vec![], Board::new(10, 10));

    let building = Building::new(
        pos(3, 3),
        BuildingKind::Mine(Mine::new(Rotation::Left, ResourcePipe::default())),
    );

    add_building(&mut sim, building).unwrap();

    let id = Id(0);
    let mut expected = Board::new(10, 10);
    expected[pos(3, 3)] = Some(Cell::inert(id));
    expected[pos(3, 4)] = Some(Cell::inert(id));
    expected[pos(4, 3)] = Some(Cell::inert(id));
    expected[pos(4, 4)] = Some(Cell::inert(id));
    expected[pos(4, 5)] = Some(Cell::input(id));
    expected[pos(4, 2)] = Some(Cell::output(id));
    assert_eq!(sim.board, expected);
}

#[test]
fn deserialize_task_001() {
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

#[test]
fn deserialize_solution_001() {
    let file = File::open("../examples/001.solution.json").unwrap();
    let serialized: Vec<dto::Object> = serde_json::from_reader(file).unwrap();
    let objects: Vec<dto::Object> = vec![
        { Object::new(dto::ObjectKind::Combiner, 9, 1, 3, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 11, 0, 4, None, None) },
        { Object::new(dto::ObjectKind::Mine, 19, 3, 2, None, None) },
        { Object::new(dto::ObjectKind::Factory, 13, 3, 0, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 14, 1, 1, None, None) },
        { Object::new(dto::ObjectKind::Mine, 5, 12, 3, None, None) },
        { Object::new(dto::ObjectKind::Mine, 6, 5, 0, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 9, 4, 7, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 7, 10, 4, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 10, 8, 7, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 10, 4, 7, None, None) },
    ];
    assert_eq!(serialized, objects)
}

#[test]
fn serialize_solution() {
    let objects: Vec<dto::Object> = vec![
        { Object::new(dto::ObjectKind::Combiner, 9, 1, 3, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 11, 0, 4, None, None) },
        { Object::new(dto::ObjectKind::Mine, 19, 3, 2, None, None) },
        { Object::new(dto::ObjectKind::Factory, 13, 3, 0, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 14, 1, 1, None, None) },
        { Object::new(dto::ObjectKind::Mine, 5, 12, 3, None, None) },
        { Object::new(dto::ObjectKind::Mine, 6, 5, 0, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 9, 4, 7, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 7, 10, 4, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 10, 8, 7, None, None) },
        { Object::new(dto::ObjectKind::Conveyor, 10, 4, 7, None, None) },
    ];

    let file = File::create("../target/test.json").unwrap();
    serde_json::to_writer(file, &objects).unwrap();

    let file = File::open("../target/test.json").unwrap();
    let serialized: Vec<dto::Object> = serde_json::from_reader(file).unwrap();
    assert_eq!(serialized, objects)
}
