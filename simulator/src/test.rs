use std::array;
use std::fs::File;

use crate::{
    dto::{self, Object},
    sim::*,
};

#[test]
fn place_mine_rotated_up() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, Board::new(10, 10));

    let building = Building::new(pos(3, 3), BuildingKind::Mine(Mine::new(Rotation::Up)));

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
    let mut sim = Sim::new(products, Board::new(10, 10));

    let building = Building::new(pos(3, 3), BuildingKind::Mine(Mine::new(Rotation::Right)));

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
    let mut sim = Sim::new(products, Board::new(10, 10));

    let building = Building::new(pos(3, 3), BuildingKind::Mine(Mine::new(Rotation::Down)));

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
    let mut sim = Sim::new(products, Board::new(10, 10));

    let building = Building::new(pos(3, 3), BuildingKind::Mine(Mine::new(Rotation::Left)));

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
fn deposit_mine_factory() {
    let mut products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    products[0] = Product::new(Resources::new([7, 0, 0, 0, 0, 0, 0, 0]), 9);

    let mut sim = Sim::new(products, Board::new(20, 10));

    add_building(
        &mut sim,
        Building::new(
            pos(0, 0),
            BuildingKind::Deposit(Deposit::new(ResourceType::Type0, 4, 4)),
        ),
    )
    .unwrap();

    add_building(
        &mut sim,
        Building::new(pos(5, 1), BuildingKind::Mine(Mine::new(Rotation::Up))),
    )
    .unwrap();

    add_building(
        &mut sim,
        Building::new(
            pos(8, 0),
            BuildingKind::Factory(Factory::new(ProductType::Type0)),
        ),
    )
    .unwrap();

    let run = run(&mut sim, 100);
    assert_eq!(
        run,
        SimRun {
            rounds: 29,
            points: 99,
            at_turn: 28
        }
    );
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
        Object {
            kind: dto::ObjectKind::Combiner,
            subtype: 3,
            x: 9,
            y: 1,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 4,
            x: 11,
            y: 0,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 2,
            x: 19,
            y: 3,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Factory,
            subtype: 0,
            x: 13,
            y: 3,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 1,
            x: 14,
            y: 1,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 3,
            x: 5,
            y: 12,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 0,
            x: 6,
            y: 5,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 9,
            y: 4,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 4,
            x: 7,
            y: 10,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 10,
            y: 8,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 10,
            y: 4,
            width: None,
            height: None,
        },
    ];
    assert_eq!(serialized, objects)
}

#[test]
fn serialize_solution() {
    let objects: Vec<dto::Object> = vec![
        Object {
            kind: dto::ObjectKind::Combiner,
            subtype: 3,
            x: 9,
            y: 1,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 4,
            x: 11,
            y: 0,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 2,
            x: 19,
            y: 3,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Factory,
            subtype: 0,
            x: 13,
            y: 3,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 1,
            x: 14,
            y: 1,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 3,
            x: 5,
            y: 12,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 0,
            x: 6,
            y: 5,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 9,
            y: 4,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 4,
            x: 7,
            y: 10,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 10,
            y: 8,
            width: None,
            height: None,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 10,
            y: 4,
            width: None,
            height: None,
        },
    ];

    let file = File::create("../target/test.json").unwrap();
    serde_json::to_writer(file, &objects).unwrap();

    let file = File::open("../target/test.json").unwrap();
    let serialized: Vec<dto::Object> = serde_json::from_reader(file).unwrap();
    assert_eq!(serialized, objects)
}
