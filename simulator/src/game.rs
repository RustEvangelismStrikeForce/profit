use core::fmt;
use std::fmt::Write;

use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Building {
    pub x: u8,
    pub y: u8,
    pub kind: BuildingKind,
}

impl Building {
    pub fn new(x: u8, y: u8, kind: BuildingKind) -> Self {
        Self { x, y, kind }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildingKind {
    Deposit(Deposit),
    Obstacle(Obstacle),
    Mine(Mine),
    Conveyor(Conveyor),
    Combiner(Combiner),
    Factory(Factory),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deposit {
    pub subtype: u8, //0..7 determines resources
    pub width: u8,
    pub height: u8,
    pub resources: Vec<u16>,
}

impl Deposit {
    pub fn new(subtype: u8, width: u8, height: u8, resources: Vec<u16>) -> Self {
        let deposit = Deposit {
            subtype,
            width,
            height,
            resources,
        };
        deposit
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Obstacle {
    pub width: u8,
    pub height: u8,
}

impl Obstacle {
    pub fn new(width: u8, height: u8) -> Self {
        let obstacle = Obstacle { width, height };
        obstacle
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mine {
    pub rotation: Rotation,
    pub resources: Vec<u16>,
}

impl Mine {
    pub fn new(rotation: Rotation, resources: Vec<u16>) -> Self {
        Self {
            rotation,
            resources,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conveyor {
    pub subtype: u8, //0..7 0..3 are different rotations of length 3 4..7 are different rotations of
    //length 4
    pub resources: u16,
}

impl Conveyor {
    pub fn new(subtype: u8, resources: u16) -> Self {
        let conveyor = Conveyor { subtype, resources };
        conveyor
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combiner {
    pub subtype: u8, //0..3 determines rotation
    pub resources: Vec<u16>,
}

impl Combiner {
    pub fn new(subtype: u8, resources: Vec<u16>) -> Self {
        let combiner = Combiner { subtype, resources };
        combiner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Factory {
    pub subtype: u8, //0..7 determines produced Product
    pub resources: Vec<u16>,
}

impl Factory {
    pub fn new(subtype: u8, resources: Vec<u16>) -> Self {
        let factory = Factory { subtype, resources };
        factory
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Product {
    pub product_type: ProductType,
    pub resources: Resouces,
    pub points: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductType {
    Type0,
    Type1,
    Type2,
    Type3,
    Type4,
    Type5,
    Type6,
    Type7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resouces {
    values: [u16; 8],
}

impl Resouces {
    pub fn new(values: [u16; 8]) -> Self {
        Self { values }
    }
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

    pub fn input(id: Id) -> Self {
        Self {
            cell_type: CellType::Input,
            id,
        }
    }

    pub fn output(id: Id) -> Self {
        Self {
            cell_type: CellType::Output,
            id,
        }
    }

    pub fn inert(id: Id) -> Self {
        Self {
            cell_type: CellType::Inert,
            id,
        }
    }

    pub fn mine(id: Id) -> [[Option<Cell>; 4]; 4] {
        fn i(id: Id) -> Option<Cell> {
            Some(Cell::new(CellType::Input, id))
        }
        fn o(id: Id) -> Option<Cell> {
            Some(Cell::new(CellType::Output, id))
        }
        fn n(id: Id) -> Option<Cell> {
            Some(Cell::new(CellType::Inert, id))
        }

        [
            [None, None, None, None],
            [None, n(id), n(id), None],
            [i(id), n(id), n(id), o(id)],
            [None, None, None, None],
        ]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Id(pub i16);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CellType {
    Input,
    Output,
    Inert,
}

#[derive(PartialEq, Eq)]
pub struct Board {
    width: u8,
    height: u8,
    board: Vec<Option<Cell>>,
}

impl std::ops::Index<(u8, u8)> for Board {
    type Output = Option<Cell>;

    fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
        assert!(
            x < self.width && y < self.height,
            "Board index out of bounds"
        );

        &self.board[(y * self.width + x) as usize]
    }
}

impl std::ops::IndexMut<(u8, u8)> for Board {
    fn index_mut(&mut self, (x, y): (u8, u8)) -> &mut Self::Output {
        assert!(
            x < self.width && y < self.height,
            "Board index out of bounds"
        );

        &mut self.board[(y * self.width + x) as usize]
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.width {
            for x in 0..self.height {
                match self[(x, y)] {
                    Some(c) => match c.cell_type {
                        CellType::Input => write!(f, "+ ")?,
                        CellType::Output => write!(f, "- ")?,
                        CellType::Inert => write!(f, "x ")?,
                    },
                    None => write!(f, ". ",)?,
                }
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}

impl Board {
    ///Standard Self new method
    ///return empty board of size width x height
    pub fn new(width: u8, height: u8) -> Self {
        Board {
            width,
            height,
            board: vec![None; (width * height) as usize],
        }
    }

    pub fn place_building(&mut self, building: &Building, id: Id) -> crate::Result<()> {
        match &building.kind {
            BuildingKind::Deposit(deposit) => {
                for i in 0..deposit.width {
                    for j in 0..deposit.height {
                        self.place_cell(
                            (building.x + i) as i8,
                            (building.y + j) as i8,
                            Cell::output(id),
                        )?;
                    }
                }
            }
            BuildingKind::Obstacle(obstacle) => {
                for i in 0..obstacle.width {
                    for j in 0..obstacle.height {
                        self.place_cell(
                            (building.x + i) as i8,
                            (building.y + j) as i8,
                            Cell::output(id),
                        )?;
                    }
                }
            }
            BuildingKind::Mine(mine) => {
                let mines = Cell::mine(id);
                for y in 0..mines.len() {
                    for x in 0..mines[0].len() {
                        if let Some(cell) = index_rotated(&mines, x, y, mine.rotation) {
                            self.place_cell(
                                building.x as i8 + x as i8 - 1,
                                building.y as i8 + y as i8 - 1,
                                cell,
                            )?;
                        }
                    }
                }
            }
            BuildingKind::Conveyor(_) => todo!(),
            BuildingKind::Combiner(_) => todo!(),
            BuildingKind::Factory(_) => todo!(),
        }

        Ok(())
    }

    fn place_cell(&mut self, x: i8, y: i8, cell: Cell) -> crate::Result<()> {
        if y < 0 || y >= self.height as i8 {
            return Err(Error::OutOfBounds);
        }
        if x < 0 || x >= self.width as i8 {
            return Err(Error::OutOfBounds);
        }

        self[(x as u8, y as u8)] = Some(cell);

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        Rotation::Left => board[x][SIZE - 1 - y],
    }
}
