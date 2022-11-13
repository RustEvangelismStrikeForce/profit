use core::fmt;
use std::fmt::Write;
use std::{string, vec};
mod dto;

struct Building {
    x: usize,
    y: usize,
    Kind: BuildingKind,
}

impl Building {
    fn new(x: usize, y: usize, Kind: BuildingKind) -> Self {
        Self { x, y, Kind }
    }
}

enum BuildingKind {
    Deposit(Deposit),
    Obstacle(Obstacle),
    Mine(Mine),
    Conveyor(Conveyor),
    Combiner(Combiner),
    Factory(Factory),
}

#[derive(Clone)]
struct Deposit {
    subtype: u8, //0..7 determines resources
    width: usize,
    height: usize,
    resources: Vec<u16>,
}

impl Deposit {
    pub fn new(subtype: u8, width: usize, height: usize, resources: Vec<u16>) -> Self {
        let deposit = Deposit {
            subtype,
            width,
            height,
            resources,
        };
        deposit
    }
}

struct Obstacle {
    width: usize,
    height: usize,
}

impl Obstacle {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        let obstacle = Obstacle { width, height };
        obstacle
    }
}

struct Mine {
    rotation: Rotation,
    resources: Vec<u16>,
}

impl Mine {
    fn new(rotation: Rotation, resources: Vec<u16>) -> Self {
        Self {
            rotation,
            resources,
        }
    }
}

struct Conveyor {
    subtype: u8, //0..7 0..3 are different rotations of length 3 4..7 are different rotations of
    //length 4
    resources: u16,
}

impl Conveyor {
    pub fn new(subtype: u8, x: usize, y: usize, resources: u16) -> Self {
        let conveyor = Conveyor { subtype, resources };
        conveyor
    }
}

struct Combiner {
    subtype: u8, //0..3 determines rotation
    resources: Vec<u16>,
}

impl Combiner {
    pub fn new(subtype: u8, x: usize, y: usize, resources: Vec<u16>) -> Self {
        let combiner = Combiner { subtype, resources };
        combiner
    }
}

struct Factory {
    subtype: u8, //0..7 determines produced Product
    resources: Vec<u16>,
}

impl Factory {
    pub fn new(subtype: u8, x: usize, y: usize, resources: Vec<u16>) -> Self {
        let factory = Factory { subtype, resources };
        factory
    }
}

struct Product {
    product_type: u8, //0..7
    resources: Vec<u8>,
    points: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    cell_type: CellType,
    id: Id,
}

impl Cell {
    pub const fn new(cell_type: CellType, id: Id) -> Self {
        Self { cell_type, id }
    }

    pub fn mine(id: Id) -> [[Option<Cell>; 4]; 4] {
        fn n(id: Id) -> Option<Cell> {
            Some(Cell::new(CellType::Inert, id))
        }
        fn i(id: Id) -> Option<Cell> {
            Some(Cell::new(CellType::Input, id))
        }
        fn o(id: Id) -> Option<Cell> {
            Some(Cell::new(CellType::Output, id))
        }

        [
            [None, None, None, None],
            [None, n(id), n(id), None],
            [i(id), n(id), n(id), o(id)],
            [None, None, None, None],
        ]
    }
}

#[derive(Clone, Copy)]
pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
}

pub fn index_rotated<const SIZE: usize>(
    board: &[[Option<Cell>; SIZE]; SIZE],
    x: usize,
    y: usize,
    rotation: Rotation,
) -> Option<Cell> {
    match rotation {
        Rotation::Up => board[y][x],
        Rotation::Right => board[SIZE - 1 - x][y],
        Rotation::Down => board[SIZE - 1 - y][SIZE - 1 - x],
        Rotation::Left => board[SIZE - 1 - x][y],
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Id(i16);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CellType {
    Input,
    Output,
    Inert,
}

#[derive(PartialEq, Eq)]
pub struct Board {
    width: usize,
    height: usize,
    board: [[Option<Cell>; 100]; 100],
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.width {
            for x in 0..self.height {
                match self.board[y][x] {
                    Some(c) => match c.cell_type {
                        CellType::Input => write!(f, "in ")?,
                        CellType::Output => write!(f, "out")?,
                        CellType::Inert => write!(f, " x ")?,
                    },
                    None => write!(f, " . ",)?,
                }
                f.write_str(" ")?;
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}

impl Board {
    ///Standard Self new method
    ///return empty board of size width x height
    pub fn new(width: usize, height: usize) -> Self {
        Board {
            width,
            height,
            board: [[None; 100]; 100],
        }
    }

    fn place_building(&mut self, building: &Building, id: Id) {
        match &building.Kind {
            BuildingKind::Deposit(deposit) => {
                for i in 0..deposit.width {
                    for j in 0..deposit.height {
                        self.board[building.x + i][building.y + j] =
                            Some(Cell::new(CellType::Output, id));
                    }
                }
            }
            BuildingKind::Obstacle(obstacle) => {
                for i in 0..obstacle.width {
                    for j in 0..obstacle.height {
                        self.board[building.x + i][building.y + j] =
                            Some(Cell::new(CellType::Inert, id));
                    }
                }
            }
            BuildingKind::Mine(mine) => {
                let mines = Cell::mine(id);
                for y in 0..mines.len() {
                    for x in 0..mines[0].len() {
                        if let Some(c) = index_rotated(&mines, x, y, mine.rotation) {
                            self.board[building.y + y - 1][building.x + x - 1] = Some(c);
                        }
                    }
                }
            }
            BuildingKind::Conveyor(_) => todo!(),
            BuildingKind::Combiner(_) => todo!(),
            BuildingKind::Factory(_) => todo!(),
        }
    }
}

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn place_mine_rotated_up() {
        let id = Id(-1);
        let mut board = Board::new(10, 10);
        let building = Building::new(3, 3, BuildingKind::Mine(Mine::new(Rotation::Up, vec![])));

        board.place_building(&building, id);

        let mut expected = Board::new(10, 10);
        expected.board[building.y][building.x] = Some(Cell::new(CellType::Inert, id));
        expected.board[building.y][building.x + 1] = Some(Cell::new(CellType::Inert, id));
        expected.board[building.y + 1][building.x] = Some(Cell::new(CellType::Inert, id));
        expected.board[building.y + 1][building.x + 1] = Some(Cell::new(CellType::Inert, id));
        expected.board[building.y + 1][building.x - 1] = Some(Cell::new(CellType::Input, id));
        expected.board[building.y + 1][building.x + 2] = Some(Cell::new(CellType::Output, id));
        assert_eq!(board, expected);
    }

    #[test]
    fn place_mine_rotated_right() {
        let id = Id(-1);
        let mut board = Board::new(10, 10);
        let building = Building::new(3, 3, BuildingKind::Mine(Mine::new(Rotation::Right, vec![])));

        board.place_building(&building, id);

        let mut expected = Board::new(10, 10);
        expected.board[building.y][building.x] = Some(Cell::new(CellType::Inert, id));
        expected.board[building.y][building.x + 1] = Some(Cell::new(CellType::Inert, id));
        expected.board[building.y + 1][building.x] = Some(Cell::new(CellType::Inert, id));
        expected.board[building.y + 1][building.x + 1] = Some(Cell::new(CellType::Inert, id));
        expected.board[building.y - 1][building.x] = Some(Cell::new(CellType::Input, id));
        expected.board[building.y + 2][building.x] = Some(Cell::new(CellType::Output, id));
        assert_eq!(board, expected);
    }
}
