use std::fs::File;

use crate::{
    dto::{self, Object},
    sim::*,
};

#[test]
fn place_mine_rotated_up() {
    let mut sim = Sim::new(Products::default(), Board::new(10, 10));

    let building = Building::mine((3, 3), Rotation::Up);
    place_building(&mut sim, building).unwrap();

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
    let mut sim = Sim::new(Products::default(), Board::new(10, 10));

    let building = Building::mine((3, 3), Rotation::Right);
    place_building(&mut sim, building).unwrap();

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
    let mut sim = Sim::new(Products::default(), Board::new(10, 10));

    let building = Building::mine((3, 3), Rotation::Down);
    place_building(&mut sim, building).unwrap();

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
    let mut sim = Sim::new(Products::default(), Board::new(10, 10));

    let building = Building::mine((3, 3), Rotation::Left);
    place_building(&mut sim, building).unwrap();

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
    let mut products = Products::default();
    products[0] = Product::new(Resources::new([7, 0, 0, 0, 0, 0, 0, 0]), 9);

    let mut sim = Sim::new(products, Board::new(20, 10));

    let building = Building::deposit((0, 0), ResourceType::Type0, 4, 4);
    place_building(&mut sim, building).unwrap();

    let building = Building::mine((5, 1), Rotation::Up);
    place_building(&mut sim, building).unwrap();

    let building = Building::factory((8, 0), ProductType::Type0);
    place_building(&mut sim, building).unwrap();

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
                    width: 5,
                    height: 5,
                },
                dto::Object {
                    kind: dto::ObjectKind::Deposit,
                    subtype: 1,
                    x: 1,
                    y: 14,
                    width: 5,
                    height: 5,
                },
                dto::Object {
                    kind: dto::ObjectKind::Deposit,
                    subtype: 2,
                    x: 22,
                    y: 1,
                    width: 7,
                    height: 7,
                },
                dto::Object {
                    kind: dto::ObjectKind::Obstacle,
                    subtype: 0,
                    x: 11,
                    y: 9,
                    width: 19,
                    height: 2,
                },
                dto::Object {
                    kind: dto::ObjectKind::Obstacle,
                    subtype: 0,
                    x: 11,
                    y: 1,
                    width: 2,
                    height: 8,
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
fn convert_task_001_to_sim() {
    let file = File::open("../examples/001.task.json").unwrap();
    let task: dto::Task = serde_json::from_reader(file).unwrap();
    let sim = Sim::try_from(&task).unwrap();

    let expected = {
        let mut products = Products::default();
        products[0] = Product::new(Resources::new([3, 3, 3, 0, 0, 0, 0, 0]), 10);
        let mut sim = Sim::new(products, Board::new(30, 20));

        let building = Building::deposit((1, 1), ResourceType::Type0, 5, 5);
        place_building(&mut sim, building).unwrap();

        let building = Building::deposit((1, 14), ResourceType::Type1, 5, 5);
        place_building(&mut sim, building).unwrap();

        let building = Building::deposit((22, 1), ResourceType::Type2, 7, 7);
        place_building(&mut sim, building).unwrap();

        let building = Building::obstacle((11, 9), 19, 2);
        place_building(&mut sim, building).unwrap();

        let building = Building::obstacle((11, 1), 2, 8);
        place_building(&mut sim, building).unwrap();

        sim
    };

    assert_eq!(sim, expected);
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
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 4,
            x: 11,
            y: 0,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 2,
            x: 19,
            y: 3,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Factory,
            subtype: 0,
            x: 13,
            y: 3,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 1,
            x: 14,
            y: 1,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 3,
            x: 5,
            y: 12,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 0,
            x: 6,
            y: 5,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 9,
            y: 4,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 4,
            x: 7,
            y: 10,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 10,
            y: 8,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 10,
            y: 4,
            width: 0,
            height: 0,
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
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 4,
            x: 11,
            y: 0,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 2,
            x: 19,
            y: 3,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Factory,
            subtype: 0,
            x: 13,
            y: 3,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 1,
            x: 14,
            y: 1,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 3,
            x: 5,
            y: 12,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Mine,
            subtype: 0,
            x: 6,
            y: 5,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 9,
            y: 4,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 4,
            x: 7,
            y: 10,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 10,
            y: 8,
            width: 0,
            height: 0,
        },
        Object {
            kind: dto::ObjectKind::Conveyor,
            subtype: 7,
            x: 10,
            y: 4,
            width: 0,
            height: 0,
        },
    ];

    let file = File::create("../target/test.json").unwrap();
    serde_json::to_writer(file, &objects).unwrap();

    let file = File::open("../target/test.json").unwrap();
    let serialized: Vec<dto::Object> = serde_json::from_reader(file).unwrap();
    assert_eq!(serialized, objects)
}

#[test]
fn run_002_solution_001() {
    let input = std::fs::read_to_string("../examples/002.solution.001.json").unwrap();
    let task: dto::Task = serde_json::from_str(&input).unwrap();
    let mut sim = Sim::try_from(&task).unwrap();
    let run = run(&mut sim, task.turns);
    assert_eq!(
        run,
        SimRun {
            rounds: 14,
            points: 20,
            at_turn: 14,
        }
    );
}

#[test]
fn run_002_solution_002() {
    let input = std::fs::read_to_string("../examples/002.solution.002.json").unwrap();
    let task: dto::Task = serde_json::from_str(&input).unwrap();
    let mut sim = Sim::try_from(&task).unwrap();
    let run = run(&mut sim, task.turns);
    assert_eq!(
        run,
        SimRun {
            rounds: 14,
            points: 60,
            at_turn: 14,
        }
    );
}
