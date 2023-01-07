use std::fs::File;

use crate::dto::{self, Object};
use crate::*;

const TURNS: u32 = 100;
const TIME: u32 = 100;

#[test]
fn place_mine_rotated_up() {
    let mut sim = Sim::new(Products::default(), Board::new(10, 10), TURNS, TIME);

    let building = Building::Mine(Mine::new((3, 3), Rotation::Up));
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
    let mut sim = Sim::new(Products::default(), Board::new(10, 10), TURNS, TIME);

    let building = Building::Mine(Mine::new((3, 3), Rotation::Right));
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
    let mut sim = Sim::new(Products::default(), Board::new(10, 10), TURNS, TIME);

    let building = Building::Mine(Mine::new((3, 3), Rotation::Down));
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
    let mut sim = Sim::new(Products::default(), Board::new(10, 10), TURNS, TIME);

    let building = Building::Mine(Mine::new((3, 3), Rotation::Left));
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
fn failing_to_place_conveyor_doesnt_remove_part_of_existing_conveyor_1() {
    let mut sim = Sim::new(Products::default(), Board::new(10, 10), TURNS, TIME);

    let building = Building::Conveyor(Conveyor::new((4, 4), Rotation::Up, false));
    place_building(&mut sim, building).unwrap();

    let building = Building::Conveyor(Conveyor::new((4, 4), Rotation::Down, true));
    place_building(&mut sim, building).unwrap_err();

    let id = Id(0);
    let mut expected = Board::new(10, 10);
    expected[pos(3, 4)] = Some(Cell::input(id));
    expected[pos(4, 4)] = Some(Cell::inert(id));
    expected[pos(5, 4)] = Some(Cell::output(id));
    assert_eq!(sim.board, expected);
}

#[test]
fn failing_to_place_conveyor_doesnt_remove_part_of_existing_conveyor_2() {
    let mut sim = Sim::new(Products::default(), Board::new(20, 20), TURNS, TIME);

    let building = Building::Conveyor(Conveyor::new((13, 15), Rotation::Left, false));
    place_building(&mut sim, building).unwrap();
    let building = Building::Conveyor(Conveyor::new((14, 15), Rotation::Right, false));
    place_building(&mut sim, building).unwrap();

    let expected = sim.board.clone();

    let building = Building::Conveyor(Conveyor::new((13, 15), Rotation::Down, false));
    place_building(&mut sim, building).unwrap_err();

    assert_eq!(sim.board, expected);
}

#[test]
fn place_conveyor_removes_part_of_existing_conveyor() {
    let mut sim = Sim::new(Products::default(), Board::new(40, 40), TURNS, TIME);

    let building = Building::Conveyor(Conveyor::new((22, 35), Rotation::Down, true));
    place_building(&mut sim, building).unwrap();
    let building = Building::Conveyor(Conveyor::new((18, 35), Rotation::Down, true));
    place_building(&mut sim, building).unwrap();
    let building = Building::Conveyor(Conveyor::new((16, 33), Rotation::Left, true));
    place_building(&mut sim, building).unwrap();
    let building = Building::Conveyor(Conveyor::new((19, 35), Rotation::Left, true));
    place_building_unchecked(&mut sim, building);

    let expected = sim.board.clone();

    let building = Building::Conveyor(Conveyor::new((18, 35), Rotation::Right, false));
    let id = place_building_unchecked(&mut sim, building);
    remove_building(&mut sim, id);

    assert_eq!(sim.board, expected);
}

#[test]
fn deposit_mine_factory() {
    let mut products = Products::default();
    products[ProductType::Type0] = Product::new(Resources::new([7, 0, 0, 0, 0, 0, 0, 0]), 9);

    let mut sim = Sim::new(products, Board::new(20, 10), TURNS, TIME);

    let building = Building::Deposit(Deposit::new((0, 0), 4, 4, ResourceType::Type0));
    place_building(&mut sim, building).unwrap();

    let building = Building::Mine(Mine::new((5, 1), Rotation::Up));
    place_building(&mut sim, building).unwrap();

    let building = Building::Factory(Factory::new((8, 0), ProductType::Type0));
    place_building(&mut sim, building).unwrap();

    let run = run(&sim);
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
fn two_ingresses_at_one_egress() {
    let mut sim = Sim::new(Products::default(), Board::new(10, 10), TURNS, TIME);

    let building = Building::Deposit(Deposit::new((0, 0), 2, 2, ResourceType::Type0));
    place_building(&mut sim, building).unwrap();

    let building = Building::Mine(Mine::new((3, 0), Rotation::Up));
    place_building(&mut sim, building).unwrap();

    let building = Building::Mine(Mine::new((1, 3), Rotation::Right));
    let res = place_building(&mut sim, building);

    println!("{:?}", sim.board);

    assert_eq!(res, Err(Error::MultipleIngresses(pos(1, 1))));
}

#[test]
fn deserialize_task_001() {
    let file = File::open("../tasks/001/task.json").unwrap();
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
    let file = File::open("../tasks/001/task.json").unwrap();
    let task: dto::Task = serde_json::from_reader(file).unwrap();
    let sim = Sim::try_from(&task).unwrap();

    let expected = {
        let mut products = Products::default();
        products[ProductType::Type0] = Product::new(Resources::new([3, 3, 3, 0, 0, 0, 0, 0]), 10);
        let mut sim = Sim::new(products, Board::new(30, 20), 50, 300);

        let building = Building::Deposit(Deposit::new((1, 1), 5, 5, ResourceType::Type0));
        place_building(&mut sim, building).unwrap();

        let building = Building::Deposit(Deposit::new((1, 14), 5, 5, ResourceType::Type1));
        place_building(&mut sim, building).unwrap();

        let building = Building::Deposit(Deposit::new((22, 1), 7, 7, ResourceType::Type2));
        place_building(&mut sim, building).unwrap();

        let building = Building::Obstacle(Obstacle::new((11, 9), 19, 2));
        place_building(&mut sim, building).unwrap();

        let building = Building::Obstacle(Obstacle::new((11, 1), 2, 8));
        place_building(&mut sim, building).unwrap();

        sim
    };

    assert_eq!(sim, expected);
}

#[test]
fn deserialize_solution_001() {
    let file = File::open("../solutions/001/solution.json").unwrap();
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
fn run_task_002_solution_001() {
    let run_task_002_solution_002 =
        std::fs::read_to_string("../tasks/002/solution_001.json").unwrap();
    let task: dto::Task = serde_json::from_str(&run_task_002_solution_002).unwrap();
    let sim = Sim::try_from(&task).unwrap();
    let run = run(&sim);
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
    let input = std::fs::read_to_string("../tasks/002/solution_002.json").unwrap();
    let task: dto::Task = serde_json::from_str(&input).unwrap();
    let sim = Sim::try_from(&task).unwrap();
    let run = run(&sim);
    assert_eq!(
        run,
        SimRun {
            rounds: 14,
            points: 60,
            at_turn: 14,
        }
    );
}

#[test]
fn reached_score_at_turn_9() {
    let input = std::fs::read_to_string("../tasks/005/task.json").unwrap();
    let task: dto::Task = serde_json::from_str(&input).unwrap();
    let sim = Sim::try_from(&task).unwrap();

    let run = run(&sim);
    assert_eq!(
        run,
        SimRun {
            rounds: 100,
            points: 20,
            at_turn: 9,
        },
    );
}
