use crate::game::*;

#[test]
fn place_mine_rotated_up() {
    let id = Id(-1);
    let mut board = Board::new(10, 10);
    let building = Building::new(3, 3, BuildingKind::Mine(Mine::new(Rotation::Up, vec![])));

    board.place_building(&building, id).unwrap();

    let mut expected = Board::new(10, 10);
    expected[(building.x, building.y)] = Some(Cell::inert(id));
    expected[(building.x + 1, building.y)] = Some(Cell::inert(id));
    expected[(building.x, building.y + 1)] = Some(Cell::inert(id));
    expected[(building.x + 1, building.y + 1)] = Some(Cell::inert(id));
    expected[(building.x - 1, building.y + 1)] = Some(Cell::input(id));
    expected[(building.x + 2, building.y + 1)] = Some(Cell::output(id));
    assert_eq!(board, expected);
}

#[test]
fn place_mine_rotated_right() {
    let id = Id(-1);
    let mut board = Board::new(10, 10);
    let building = Building::new(3, 3, BuildingKind::Mine(Mine::new(Rotation::Right, vec![])));

    board.place_building(&building, id).unwrap();

    let mut expected = Board::new(10, 10);
    expected[(building.x, building.y)] = Some(Cell::inert(id));
    expected[(building.x + 1, building.y)] = Some(Cell::inert(id));
    expected[(building.x, building.y + 1)] = Some(Cell::inert(id));
    expected[(building.x + 1, building.y + 1)] = Some(Cell::inert(id));
    expected[(building.x, building.y - 1)] = Some(Cell::input(id));
    expected[(building.x, building.y + 2)] = Some(Cell::output(id));
    assert_eq!(board, expected);
}

#[test]
fn place_mine_rotated_down() {
    let id = Id(-1);
    let mut board = Board::new(10, 10);
    let building = Building::new(3, 3, BuildingKind::Mine(Mine::new(Rotation::Down, vec![])));

    board.place_building(&building, id).unwrap();

    let mut expected = Board::new(10, 10);
    expected[(building.x, building.y)] = Some(Cell::inert(id));
    expected[(building.x + 1, building.y)] = Some(Cell::inert(id));
    expected[(building.x, building.y + 1)] = Some(Cell::inert(id));
    expected[(building.x + 1, building.y + 1)] = Some(Cell::inert(id));
    expected[(building.x + 2, building.y)] = Some(Cell::input(id));
    expected[(building.x - 1, building.y)] = Some(Cell::output(id));
    assert_eq!(board, expected);
}

#[test]
fn place_mine_rotated_left() {
    let id = Id(-1);
    let mut board = Board::new(10, 10);
    let building = Building::new(3, 3, BuildingKind::Mine(Mine::new(Rotation::Left, vec![])));

    board.place_building(&building, id).unwrap();

    let mut expected = Board::new(10, 10);
    expected[(building.x, building.y)] = Some(Cell::inert(id));
    expected[(building.x + 1, building.y)] = Some(Cell::inert(id));
    expected[(building.x, building.y + 1)] = Some(Cell::inert(id));
    expected[(building.x + 1, building.y + 1)] = Some(Cell::inert(id));
    expected[(building.x + 1, building.y + 2)] = Some(Cell::input(id));
    expected[(building.x + 1, building.y - 1)] = Some(Cell::output(id));
    assert_eq!(board, expected);
}
