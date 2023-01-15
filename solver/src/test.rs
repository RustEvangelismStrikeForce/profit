use std::cmp::Ordering;

use sim::{dto, place_building, pos, Board, Building, Id, Obstacle, Products, Sim};

use crate::{find_regions, Regions};

const TURNS: u32 = 100;
const TIME: f32 = 100.0;

#[test]
fn find_two_regions() {
    let mut sim = Sim::new(Products::default(), Board::new(6, 6), TURNS, TIME);
    place_building(&mut sim, Building::Obstacle(Obstacle::new(pos(3, 0), 1, 6))).unwrap();

    let mut regions = find_regions(&sim);
    for i in 0..regions.len() {
        let r = regions.get_mut(i);
        r.cells.sort_by(|a, b| match a.y.cmp(&b.y) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => a.x.cmp(&b.x),
        });
    }

    #[rustfmt::skip]
    let expected = Regions {
        deposits: vec![],
        cells: vec![
            // first
            pos(0, 0), pos(1, 0), pos(2, 0),
            pos(0, 1), pos(1, 1), pos(2, 1),
            pos(0, 2), pos(1, 2), pos(2, 2),
            pos(0, 3), pos(1, 3), pos(2, 3),
            pos(0, 4), pos(1, 4), pos(2, 4),
            pos(0, 5), pos(1, 5), pos(2, 5),
            // second
            pos(4, 0), pos(5, 0),
            pos(4, 1), pos(5, 1),
            pos(4, 2), pos(5, 2),
            pos(4, 3), pos(5, 3),
            pos(4, 4), pos(5, 4),
            pos(4, 5), pos(5, 5),
        ],
        indices: vec![(0, 0), (0, 18)],
    };

    assert_eq!(regions, expected,);
}

#[test]
fn find_regions_of_task_002() {
    let input = std::fs::read_to_string("../tasks/002/task.json").unwrap();
    let task: dto::Task = serde_json::from_str(&input).unwrap();
    let sim = Sim::try_from(&task).unwrap();

    let mut regions = find_regions(&sim);
    for i in 0..regions.len() {
        let r = regions.get_mut(i);
        r.cells.sort_by(|a, b| match a.y.cmp(&b.y) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => a.x.cmp(&b.x),
        });
    }

    #[rustfmt::skip]
    let expected = Regions {
        deposits: vec![Id(0)],
        cells: vec![
            pos(5, 0), pos(6, 0), pos(7, 0), pos(8, 0), pos(9, 0), pos(10, 0), pos(11, 0), pos(12, 0), pos(13, 0), pos(14, 0), pos(15, 0), pos(16, 0), pos(17, 0), pos(18, 0), pos(19, 0), pos(20, 0), pos(21, 0), pos(22, 0), pos(23, 0), pos(24, 0), pos(25, 0),
            pos(5, 1), pos(6, 1), pos(7, 1), pos(8, 1), pos(9, 1), pos(10, 1), pos(11, 1), pos(12, 1), pos(13, 1), pos(14, 1), pos(15, 1), pos(16, 1), pos(17, 1), pos(18, 1), pos(19, 1), pos(20, 1), pos(21, 1), pos(22, 1), pos(23, 1), pos(24, 1), pos(25, 1),
                                                                                                                                                                                                       pos(21, 2), pos(22, 2), pos(23, 2), pos(24, 2), pos(25, 2),
            pos(5, 3), pos(6, 3), pos(7, 3), pos(8, 3), pos(9, 3), pos(10, 3), pos(11, 3), pos(12, 3), pos(13, 3), pos(14, 3), pos(15, 3), pos(16, 3), pos(17, 3), pos(18, 3), pos(19, 3), pos(20, 3), pos(21, 3), pos(22, 3), pos(23, 3), pos(24, 3), pos(25, 3),
            pos(5, 4), pos(6, 4), pos(7, 4), pos(8, 4), pos(9, 4), pos(10, 4), pos(11, 4), pos(12, 4), pos(13, 4), pos(14, 4), pos(15, 4), pos(16, 4), pos(17, 4), pos(18, 4), pos(19, 4), pos(20, 4), pos(21, 4), pos(22, 4), pos(23, 4), pos(24, 4), pos(25, 4),
        ],
        indices: vec![(0, 0)],
    };

    assert_eq!(regions, expected);
}
