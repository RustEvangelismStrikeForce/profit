use std::{string, vec};
mod dto;

struct Building {
    x: usize,
    y: usize,
    Kind: BuildingKind,
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
    subtype: u8, //0..3 determines rotation
    resources: Vec<u16>,
}

impl Mine {
    pub fn new(subtype: u8, x: usize, y: usize) -> Self {
        let mine = Mine {
            subtype,
            resources: vec![0; 8],
        };
        mine
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

#[derive(Copy, Clone)]
pub struct Cell {
    cell_type: CellType,
    id: Id,
}

impl Cell {
    pub fn new(cell_type: CellType, id: Id) -> Self {
        Self { cell_type, id }
    }
}

#[derive(Copy, Clone)]
pub struct Id(i16);

#[derive(Clone, Copy, Default)]
pub enum CellType {
    Input,
    Output,
    #[default]
    Inert,
}

pub struct Board {
    width: usize,
    height: usize,
    board: [[Option<Cell>; 100]; 100],
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
            BuildingKind::Mine(mine) => match mine.subtype {
                0 => {
                    self.board[building.x][building.y] = Some(Cell::new(CellType::Inert, id));
                    self.board[building.x + 1][building.y] = Some(Cell::new(CellType::Inert, id));
                    self.board[building.x][building.y + 1] = Some(Cell::new(CellType::Inert, id));
                    self.board[building.x + 1][building.y + 1] =
                        Some(Cell::new(CellType::Inert, id));
                    self.board[building.x - 1][building.y + 1] =
                        Some(Cell::new(CellType::Input, id));
                    self.board[building.x + 2][building.y + 1] =
                        Some(Cell::new(CellType::Output, id));
                }
                1 => {}
                2 => {}
                3 => {}
                _ => todo!(),
            },
            BuildingKind::Conveyor(_) => todo!(),
            BuildingKind::Combiner(_) => todo!(),
            BuildingKind::Factory(_) => todo!(),
        }
    }
}
fn main() {}
