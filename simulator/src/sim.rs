//! origin: #
//! inert:  o
//! input:  +
//! output: -

use core::fmt;
use std::fmt::Write;

use crate::Error;

pub const RESOURCE_TYPES: usize = 8;
pub const FACTORY_SIZE: usize = 5;

pub const MINE_CELLS: [[(Pos, CellType); 6]; 4] = [
    //
    //  #o
    // +oo-
    //
    [
        (pos(-1, 1), CellType::Input),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Inert),
        (pos(0, 1), CellType::Inert),
        (pos(1, 1), CellType::Inert),
        (pos(2, 1), CellType::Output),
    ],
    // +
    // #o
    // oo
    // -
    [
        (pos(0, -1), CellType::Input),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Inert),
        (pos(0, 1), CellType::Inert),
        (pos(1, 1), CellType::Inert),
        (pos(0, 2), CellType::Output),
    ],
    //
    // -#o+
    //  oo
    //
    [
        (pos(-1, 0), CellType::Output),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Inert),
        (pos(0, 1), CellType::Inert),
        (pos(1, 1), CellType::Inert),
        (pos(2, 0), CellType::Input),
    ],
    //  -
    // #o
    // oo
    //  +
    [
        (pos(1, 2), CellType::Input),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Inert),
        (pos(0, 1), CellType::Inert),
        (pos(1, 1), CellType::Inert),
        (pos(1, -1), CellType::Output),
    ],
];

pub const SMALL_CONVEYOR_CELLS: [[(Pos, CellType); 3]; 4] = [
    //
    // +#-
    //
    [
        (pos(-1, 0), CellType::Input),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Output),
    ],
    // +
    // #
    // -
    [
        (pos(0, -1), CellType::Input),
        (pos(0, 0), CellType::Inert),
        (pos(0, 1), CellType::Output),
    ],
    //
    // -#+
    //
    [
        (pos(-1, 0), CellType::Output),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Input),
    ],
    // -
    // #
    // +
    [
        (pos(0, -1), CellType::Output),
        (pos(0, 0), CellType::Inert),
        (pos(0, 1), CellType::Input),
    ],
];
pub const BIG_CONVEYOR_CELLS: [[(Pos, CellType); 4]; 4] = [
    //
    // +#o-
    //
    //
    [
        (pos(-1, 0), CellType::Input),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Inert),
        (pos(2, 0), CellType::Output),
    ],
    // +
    // #
    // o
    // -
    [
        (pos(0, -1), CellType::Input),
        (pos(0, 0), CellType::Inert),
        (pos(0, 1), CellType::Inert),
        (pos(0, 2), CellType::Output),
    ],
    //
    // -#o+
    //
    [
        (pos(-1, 0), CellType::Output),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Inert),
        (pos(2, 0), CellType::Input),
    ],
    // -
    // #
    // o
    // +
    [
        (pos(0, -1), CellType::Output),
        (pos(0, 0), CellType::Inert),
        (pos(0, 1), CellType::Inert),
        (pos(0, 2), CellType::Input),
    ],
];
pub const COMBINER_CELLS: [[(Pos, CellType); 7]; 4] = [
    // +o
    // +#-
    // +o
    [
        (pos(-1, -1), CellType::Input),
        (pos(-1, 0), CellType::Input),
        (pos(-1, 1), CellType::Input),
        (pos(0, -1), CellType::Inert),
        (pos(0, 0), CellType::Inert),
        (pos(0, 1), CellType::Inert),
        (pos(1, 0), CellType::Output),
    ],
    // +++
    // o#o
    //  -
    [
        (pos(-1, -1), CellType::Input),
        (pos(0, -1), CellType::Input),
        (pos(1, -1), CellType::Input),
        (pos(-1, 0), CellType::Inert),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Inert),
        (pos(0, 1), CellType::Output),
    ],
    //  o+
    // -#+
    //  o+
    [
        (pos(-1, 0), CellType::Output),
        (pos(0, -1), CellType::Inert),
        (pos(0, 0), CellType::Inert),
        (pos(0, 1), CellType::Inert),
        (pos(1, -1), CellType::Input),
        (pos(1, 0), CellType::Input),
        (pos(1, 1), CellType::Input),
    ],
    //  -
    // o#o
    // +++
    [
        (pos(0, 1), CellType::Output),
        (pos(-1, 0), CellType::Inert),
        (pos(0, 0), CellType::Inert),
        (pos(1, 0), CellType::Inert),
        (pos(-1, 1), CellType::Input),
        (pos(0, 1), CellType::Input),
        (pos(1, 1), CellType::Input),
    ],
];

pub struct Sim {
    pub products: [Product; 8],
    pub given: Vec<Building>,
    pub placed: Vec<Building>,
    pub board: Board,
}

impl Sim {
    pub fn new(
        products: [Product; 8],
        given: Vec<Building>,
        placed: Vec<Building>,
        board: Board,
    ) -> Self {
        Self {
            products,
            given,
            placed,
            board,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Building {
    pub x: i8,
    pub y: i8,
    pub kind: BuildingKind,
}

impl Building {
    pub fn new(x: i8, y: i8, kind: BuildingKind) -> Self {
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
    pub product_type: ProductType,
    pub width: u8,
    pub height: u8,
    pub resources: Resources,
}

impl Deposit {
    pub fn new(product_type: ProductType, width: u8, height: u8, resources: Resources) -> Self {
        Self {
            product_type,
            width,
            height,
            resources,
        }
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
    pub resources: ResourcePipe,
}

impl Mine {
    pub fn new(rotation: Rotation, resources: ResourcePipe) -> Self {
        Self {
            rotation,
            resources,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conveyor {
    pub rotation: Rotation,
    pub big: bool,
    pub resources: ResourcePipe,
}

impl Conveyor {
    pub fn new(rotation: Rotation, big: bool, resources: ResourcePipe) -> Self {
        Self {
            rotation,
            big,
            resources,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combiner {
    pub rotation: Rotation,
    pub resources: ResourcePipe,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Factory {
    pub product_type: ProductType,
    pub resources: Resources,
}

impl Factory {
    pub fn new(product_type: ProductType, resources: Resources) -> Self {
        Self {
            product_type,
            resources,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Product {
    pub resources: Resources,
    pub points: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductType {
    Type0 = 0,
    Type1 = 1,
    Type2 = 2,
    Type3 = 3,
    Type4 = 4,
    Type5 = 5,
    Type6 = 6,
    Type7 = 7,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourcePipe {
    pub input: Resources,
    pub output: Resources,
}

impl ResourcePipe {
    pub fn new(input: Resources, output: Resources) -> Self {
        Self { input, output }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resources {
    pub values: [u16; RESOURCE_TYPES],
}

impl Resources {
    pub fn new(values: [u16; RESOURCE_TYPES]) -> Self {
        Self { values }
    }

    pub fn clear(&mut self) {
        self.values = [0; RESOURCE_TYPES];
    }
}

#[derive(PartialEq, Eq)]
pub struct Board {
    width: i8,
    height: i8,
    cells: Vec<Option<Cell>>,
}

impl std::ops::Index<Pos> for Board {
    type Output = Option<Cell>;

    fn index(&self, pos: Pos) -> &Self::Output {
        assert!(
            pos.x < self.width && pos.y < self.height,
            "Board index out of bounds"
        );

        &self.cells[(pos.y * self.width + pos.x) as usize]
    }
}

impl std::ops::IndexMut<Pos> for Board {
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        assert!(
            pos.x < self.width && pos.y < self.height,
            "Board index out of bounds"
        );

        &mut self.cells[(pos.y * self.width + pos.x) as usize]
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.width {
            for x in 0..self.height {
                match self[pos(x, y)] {
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
    pub fn new(width: i8, height: i8) -> Self {
        Board {
            width,
            height,
            cells: vec![None; (width * height) as usize],
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pos {
    pub x: i8,
    pub y: i8,
}

impl std::ops::Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Self) -> Self::Output {
        Pos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Pos> for Pos {
    type Output = Pos;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::AddAssign<Pos> for Pos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign<Pos> for Pos {
    fn sub_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[inline(always)]
pub const fn pos(x: i8, y: i8) -> Pos {
    Pos { x, y }
}

pub fn place_building(sim: &mut Sim, building: &Building, id: Id) -> crate::Result<()> {
    let res = || -> crate::Result<()> {
        match &building.kind {
            BuildingKind::Deposit(deposit) => {
                for y in 0..deposit.height {
                    for x in 0..deposit.width {
                        place_cell(
                            sim,
                            pos(building.x + x as i8, building.y + y as i8),
                            Cell::output(id),
                        )?;
                    }
                }
            }
            BuildingKind::Obstacle(obstacle) => {
                for y in 0..obstacle.height {
                    for x in 0..obstacle.width {
                        place_cell(
                            sim,
                            pos(building.x + x as i8, building.y + y as i8),
                            Cell::output(id),
                        )?;
                    }
                }
            }
            BuildingKind::Mine(mine) => {
                for (pos, ty) in MINE_CELLS[mine.rotation as usize] {
                    place_cell(sim, pos, Cell::new(ty, id))?;
                }
            }
            BuildingKind::Conveyor(conveyor) => {
                if conveyor.big {
                    for (pos, ty) in BIG_CONVEYOR_CELLS[conveyor.rotation as usize] {
                        place_cell(sim, pos, Cell::new(ty, id))?;
                    }
                } else {
                    for (pos, ty) in SMALL_CONVEYOR_CELLS[conveyor.rotation as usize] {
                        place_cell(sim, pos, Cell::new(ty, id))?;
                    }
                }
            }
            BuildingKind::Combiner(combiner) => {
                for (pos, ty) in COMBINER_CELLS[combiner.rotation as usize] {
                    place_cell(sim, pos, Cell::new(ty, id))?;
                }
            }
            BuildingKind::Factory(_) => {
                for y in 0..FACTORY_SIZE {
                    for x in 0..FACTORY_SIZE {
                        place_cell(
                            sim,
                            pos(building.x + x as i8, building.y + y as i8),
                            Cell::output(id),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }();

    // cleanup if placing the building failed
    if res.is_err() {
        for c in sim.board.cells.iter_mut() {
            if let Some(cell) = c {
                if cell.id == id {
                    *c = None;
                }
            }
        }
    }

    res
}

fn place_cell(sim: &mut Sim, pos: Pos, cell: Cell) -> crate::Result<()> {
    if pos.y < 0 || pos.y >= sim.board.height as i8 {
        return Err(Error::OutOfBounds);
    }
    if pos.x < 0 || pos.x >= sim.board.width as i8 {
        return Err(Error::OutOfBounds);
    }

    if sim.board[pos].is_some() {
        return Err(Error::CellNotEmpty);
    }

    sim.board[pos] = Some(cell);

    Ok(())
}
