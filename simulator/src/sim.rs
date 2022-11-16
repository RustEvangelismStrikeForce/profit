//! origin: #
//! inert:  o
//! input:  +
//! output: -

use core::fmt;
use std::fmt::Write;

use crate::Error;

pub const RESOURCE_TYPES: usize = 8;
pub const FACTORY_SIZE: i8 = 5;

pub const MINE_CELLS: [[(Pos, CellKind); 6]; 4] = [
    //   # o
    // + o o -
    [
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Inert),
        (pos(-1, 1), CellKind::Input),
        (pos(0, 1), CellKind::Inert),
        (pos(1, 1), CellKind::Inert),
        (pos(2, 1), CellKind::Output),
    ],
    // +
    // # o
    // o o
    // -
    [
        (pos(0, -1), CellKind::Input),
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Inert),
        (pos(1, 1), CellKind::Inert),
        (pos(0, 2), CellKind::Output),
    ],
    // - # o +
    //   o o
    [
        (pos(-1, 0), CellKind::Output),
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Inert),
        (pos(1, 1), CellKind::Inert),
        (pos(2, 0), CellKind::Input),
    ],
    //   -
    // # o
    // o o
    //   +
    [
        (pos(1, -1), CellKind::Output),
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Inert),
        (pos(1, 1), CellKind::Inert),
        (pos(1, 2), CellKind::Input),
    ],
];
pub const ADJACENT_MINE_CELLS: [[(Pos, Pos); 6]; 4] = [
    //   ? # o ?
    // ? + o o - ?
    //   ?     ?
    [
        (pos(-1, 0), pos(-1, 1)),
        (pos(2, 0), pos(2, 1)),
        (pos(-2, 1), pos(-1, 1)),
        (pos(3, 1), pos(2, 1)),
        (pos(-1, 2), pos(-1, 1)),
        (pos(2, 2), pos(2, 1)),
    ],
    //   ?
    // ? + ?
    //   # o
    //   o o
    // ? - ?
    //   ?
    [
        (pos(0, -2), pos(0, -1)),
        (pos(-1, -1), pos(0, -1)),
        (pos(1, -1), pos(0, -1)),
        (pos(-1, 2), pos(0, 2)),
        (pos(1, 2), pos(0, 2)),
        (pos(0, 3), pos(0, 2)),
    ],
    //   ?     ?
    // ? - # o + ?
    //   ? o o ?
    [
        (pos(-1, -1), pos(-1, 0)),
        (pos(2, -1), pos(2, 0)),
        (pos(-2, 0), pos(-1, 0)),
        (pos(3, 0), pos(2, 0)),
        (pos(-1, 1), pos(-1, 0)),
        (pos(2, 1), pos(2, 0)),
    ],
    //   ?
    // ? - ?
    // # o
    // o o
    // ? + ?
    //   ?
    [
        (pos(1, -2), pos(1, -1)),
        (pos(0, -1), pos(1, -1)),
        (pos(2, -1), pos(1, -1)),
        (pos(0, 2), pos(1, 2)),
        (pos(2, 2), pos(1, 2)),
        (pos(1, 3), pos(1, 2)),
    ],
];

pub const SMALL_CONVEYOR_CELLS: [[(Pos, CellKind); 3]; 4] = [
    // + # -
    [
        (pos(-1, 0), CellKind::Input),
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Output),
    ],
    // +
    // #
    // -
    [
        (pos(0, -1), CellKind::Input),
        (pos(0, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Output),
    ],
    // - # +
    [
        (pos(-1, 0), CellKind::Output),
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Input),
    ],
    // -
    // #
    // +
    [
        (pos(0, -1), CellKind::Output),
        (pos(0, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Input),
    ],
];
pub const ADJACENT_SMALL_CONVEYOR_CELLS: [[(Pos, Pos); 6]; 4] = [
    //   ?   ?
    // ? + # - ?
    //   ?   ?
    [
        (pos(-1, -1), pos(-1, 0)),
        (pos(1, -1), pos(1, 0)),
        (pos(-2, 0), pos(-1, 0)),
        (pos(2, 0), pos(1, 0)),
        (pos(-1, 1), pos(-1, 0)),
        (pos(1, 1), pos(1, 0)),
    ],
    //   ?
    // ? + ?
    //   #
    // ? - ?
    //   ?
    [
        (pos(0, -2), pos(0, -1)),
        (pos(-1, -1), pos(0, -1)),
        (pos(1, -1), pos(0, -1)),
        (pos(-1, 1), pos(0, 1)),
        (pos(1, 1), pos(0, 1)),
        (pos(0, 2), pos(0, 1)),
    ],
    //   ?   ?
    // ? - # + ?
    //   ?   ?
    [
        (pos(-1, -1), pos(-1, 0)),
        (pos(1, -1), pos(1, 0)),
        (pos(-2, 0), pos(-1, 0)),
        (pos(2, 0), pos(1, 0)),
        (pos(-1, 1), pos(-1, 0)),
        (pos(1, 1), pos(1, 0)),
    ],
    //   ?
    // ? - ?
    //   #
    // ? + ?
    //   ?
    [
        (pos(0, -2), pos(0, -1)),
        (pos(-1, -1), pos(0, -1)),
        (pos(1, -1), pos(0, -1)),
        (pos(-1, 1), pos(0, 1)),
        (pos(1, 1), pos(0, 1)),
        (pos(0, 2), pos(0, 1)),
    ],
];

pub const BIG_CONVEYOR_CELLS: [[(Pos, CellKind); 4]; 4] = [
    // + # o -
    [
        (pos(-1, 0), CellKind::Input),
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Inert),
        (pos(2, 0), CellKind::Output),
    ],
    // +
    // #
    // o
    // -
    [
        (pos(0, -1), CellKind::Input),
        (pos(0, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Inert),
        (pos(0, 2), CellKind::Output),
    ],
    // - # o +
    [
        (pos(-1, 0), CellKind::Output),
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Inert),
        (pos(2, 0), CellKind::Input),
    ],
    // -
    // #
    // o
    // +
    [
        (pos(0, -1), CellKind::Output),
        (pos(0, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Inert),
        (pos(0, 2), CellKind::Input),
    ],
];
pub const ADJACENT_BIG_CONVEYOR_CELLS: [[(Pos, Pos); 6]; 4] = [
    //   ?     ?
    // ? + # o - ?
    //   ?     ?
    [
        (pos(-1, -1), pos(-1, 0)),
        (pos(2, -1), pos(2, 0)),
        (pos(-2, 0), pos(-1, 0)),
        (pos(3, 0), pos(2, 0)),
        (pos(-1, 1), pos(-1, 0)),
        (pos(2, 1), pos(2, 0)),
    ],
    //   ?
    // ? + ?
    //   #
    //   o
    // ? - ?
    //   ?
    [
        (pos(0, -2), pos(0, -1)),
        (pos(-1, -1), pos(0, -1)),
        (pos(1, -1), pos(0, -1)),
        (pos(-1, 2), pos(0, 2)),
        (pos(1, 2), pos(0, 2)),
        (pos(0, 3), pos(0, 2)),
    ],
    //   ?     ?
    // ? - # o + ?
    //   ?     ?
    [
        (pos(-1, -1), pos(-1, 0)),
        (pos(2, -1), pos(2, 0)),
        (pos(-2, 0), pos(-1, 0)),
        (pos(3, 0), pos(2, 0)),
        (pos(-1, 1), pos(-1, 0)),
        (pos(2, 1), pos(2, 0)),
    ],
    //   ?
    // ? - ?
    //   #
    //   o
    // ? + ?
    //   ?
    [
        (pos(0, -2), pos(0, -1)),
        (pos(-1, -1), pos(0, -1)),
        (pos(1, -1), pos(0, -1)),
        (pos(-1, 2), pos(0, 2)),
        (pos(1, 2), pos(0, 2)),
        (pos(0, 3), pos(0, 2)),
    ],
];
pub const COMBINER_CELLS: [[(Pos, CellKind); 7]; 4] = [
    // + o
    // + # -
    // + o
    [
        (pos(-1, -1), CellKind::Input),
        (pos(-1, 0), CellKind::Input),
        (pos(-1, 1), CellKind::Input),
        (pos(0, -1), CellKind::Inert),
        (pos(0, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Inert),
        (pos(1, 0), CellKind::Output),
    ],
    // + + +
    // o # o
    //   -
    [
        (pos(-1, -1), CellKind::Input),
        (pos(0, -1), CellKind::Input),
        (pos(1, -1), CellKind::Input),
        (pos(-1, 0), CellKind::Inert),
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Output),
    ],
    //   o +
    // - # +
    //   o +
    [
        (pos(-1, 0), CellKind::Output),
        (pos(0, -1), CellKind::Inert),
        (pos(0, 0), CellKind::Inert),
        (pos(0, 1), CellKind::Inert),
        (pos(1, -1), CellKind::Input),
        (pos(1, 0), CellKind::Input),
        (pos(1, 1), CellKind::Input),
    ],
    //   -
    // o # o
    // + + +
    [
        (pos(0, -1), CellKind::Output),
        (pos(-1, 0), CellKind::Inert),
        (pos(0, 0), CellKind::Inert),
        (pos(1, 0), CellKind::Inert),
        (pos(-1, 1), CellKind::Input),
        (pos(0, 1), CellKind::Input),
        (pos(1, 1), CellKind::Input),
    ],
];
pub const ADJACENT_COMBINER_CELLS: [[(Pos, Pos); 8]; 4] = [
    //   ?
    // ? + o ?
    // ? + # - ?
    // ? + o ?
    //   ?
    [
        (pos(-1, -2), pos(-1, -1)),
        (pos(-2, -1), pos(-1, -1)),
        (pos(-2, 0), pos(-1, 0)),
        (pos(-2, 1), pos(-1, 1)),
        (pos(-1, 2), pos(-1, 1)),
        (pos(1, -1), pos(1, 0)),
        (pos(2, 0), pos(1, 0)),
        (pos(1, 1), pos(1, 0)),
    ],
    //   ? ? ?
    // ? + + + ?
    //   o # o
    //   ? - ?
    //     ?
    [
        (pos(-2, -1), pos(-1, -1)),
        (pos(-1, -2), pos(-1, -1)),
        (pos(0, -2), pos(0, -1)),
        (pos(1, -2), pos(1, -1)),
        (pos(2, -1), pos(1, -1)),
        (pos(-1, 1), pos(0, 1)),
        (pos(1, 1), pos(0, 1)),
        (pos(0, 2), pos(0, 1)),
    ],
    //       ?
    //   ? o + ?
    // ? - # + ?
    //   ? o + ?
    //       ?
    [
        (pos(-1, -1), pos(-1, 0)),
        (pos(-2, 0), pos(-1, 0)),
        (pos(-1, 1), pos(-1, 0)),
        (pos(1, -2), pos(1, -1)),
        (pos(2, -1), pos(1, -1)),
        (pos(2, 0), pos(1, 0)),
        (pos(2, 1), pos(1, 1)),
        (pos(1, 2), pos(1, 1)),
    ],
    //     ?
    //   ? - ?
    //   o # o
    // ? + + + ?
    //   ? ? ?
    [
        (pos(0, -2), pos(0, -1)),
        (pos(-1, -1), pos(0, -1)),
        (pos(1, -1), pos(0, -1)),
        (pos(-2, 1), pos(-1, 1)),
        (pos(-1, 2), pos(-1, 1)),
        (pos(0, 2), pos(0, 1)),
        (pos(1, 2), pos(1, 1)),
        (pos(2, 1), pos(1, 1)),
    ],
];

pub struct Sim {
    pub products: [Product; 8],
    pub buildings: Vec<Building>,
    pub board: Board,
    pub connections: Vec<Connection>,
}

impl Sim {
    pub fn new(
        products: [Product; 8],
        buildings: Vec<Building>,
        board: Board,
        connections: Vec<Connection>,
    ) -> Self {
        Self {
            products,
            buildings,
            board,
            connections,
        }
    }

    pub fn next_id(&self) -> Id {
        Id(self.buildings.len() as u16)
    }

    pub fn building(&self, id: Id) -> &Building {
        &self.buildings[id.0 as usize]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Building {
    pub pos: Pos,
    pub kind: BuildingKind,
}

impl Building {
    pub fn new(pos: Pos, kind: BuildingKind) -> Self {
        Self { pos, kind }
    }

    pub fn take_resources(&mut self) -> Resources {
        match &mut self.kind {
            BuildingKind::Deposit(deposit) => {
                let num = deposit.resources.max(3);
                deposit.resources -= num;

                let mut res = Resources::default();
                res.values[deposit.product_type as usize] += num;
                res
            }
            BuildingKind::Obstacle(_) => panic!("Obstacles cannot contain resources"),
            BuildingKind::Mine(mine) => std::mem::take(&mut mine.resources),
            BuildingKind::Conveyor(conveyor) => std::mem::take(&mut conveyor.resources),
            BuildingKind::Combiner(combiner) => std::mem::take(&mut combiner.resources),
            BuildingKind::Factory(_) => panic!("Facotories cannot output resources"),
        }
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
    pub resources: u16,
}

impl Deposit {
    pub fn new(product_type: ProductType, width: u8, height: u8, resources: u16) -> Self {
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
    pub resources: Resources,
}

impl Mine {
    pub fn new(rotation: Rotation, resources: Resources) -> Self {
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
    pub resources: Resources,
}

impl Conveyor {
    pub fn new(rotation: Rotation, big: bool, resources: Resources) -> Self {
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
    pub resources: Resources,
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

impl Product {
    pub fn new(resources: Resources, points: u32) -> Self {
        Self { resources, points }
    }
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
pub struct Resources {
    pub values: [u16; RESOURCE_TYPES],
}

impl Resources {
    pub fn new(values: [u16; RESOURCE_TYPES]) -> Self {
        Self { values }
    }
}

#[derive(PartialEq, Eq)]
pub struct Board {
    width: i8,
    height: i8,
    cells: Vec<Option<Cell>>,
}

impl<P: Into<Pos>> std::ops::Index<P> for Board {
    type Output = Option<Cell>;

    fn index(&self, pos: P) -> &Self::Output {
        let pos = pos.into();
        assert!(
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height,
            "Board index out of bounds"
        );

        &self.cells[(pos.y * self.width + pos.x) as usize]
    }
}

impl<P: Into<Pos>> std::ops::IndexMut<P> for Board {
    fn index_mut(&mut self, pos: P) -> &mut Self::Output {
        let pos = pos.into();
        assert!(
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height,
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
                    Some(c) => match c.kind {
                        CellKind::Input => write!(f, "+ ")?,
                        CellKind::Output => write!(f, "- ")?,
                        CellKind::Inert => write!(f, "x ")?,
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

    pub fn get(&self, pos: impl Into<Pos>) -> Option<Cell> {
        let pos = pos.into();
        if pos.x < 0 || pos.x >= self.width {
            return None;
        }
        if pos.y < 0 || pos.y >= self.height {
            return None;
        }
        self[pos]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    kind: CellKind,
    id: Id,
}

impl Cell {
    pub const fn new(cell_type: CellKind, id: Id) -> Self {
        Self {
            kind: cell_type,
            id,
        }
    }

    pub fn input(id: Id) -> Self {
        Self {
            kind: CellKind::Input,
            id,
        }
    }

    pub fn output(id: Id) -> Self {
        Self {
            kind: CellKind::Output,
            id,
        }
    }

    pub fn inert(id: Id) -> Self {
        Self {
            kind: CellKind::Inert,
            id,
        }
    }

    pub fn mine(id: Id) -> [[Option<Cell>; 4]; 4] {
        fn i(id: Id) -> Option<Cell> {
            Some(Cell::new(CellKind::Input, id))
        }
        fn o(id: Id) -> Option<Cell> {
            Some(Cell::new(CellKind::Output, id))
        }
        fn n(id: Id) -> Option<Cell> {
            Some(Cell::new(CellKind::Inert, id))
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
pub struct Id(pub u16);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CellKind {
    Input,
    Output,
    Inert,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    /// Output cell - input of the connection
    pub output: Id,
    /// Input cell - output of the connection
    pub input: Id,
    pub resources: Resources,
}

impl Connection {
    pub fn new(input: Id, output: Id) -> Self {
        Self {
            input,
            output,
            resources: Resources::default(),
        }
    }
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

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<(i8, i8)> for Pos {
    fn from((x, y): (i8, i8)) -> Self {
        Pos { x, y }
    }
}

impl<P: Into<Pos>> std::ops::Add<P> for Pos {
    type Output = Pos;

    fn add(self, rhs: P) -> Self::Output {
        let rhs = rhs.into();
        Pos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<P: Into<Pos>> std::ops::Sub<P> for Pos {
    type Output = Pos;

    fn sub(self, rhs: P) -> Self::Output {
        let rhs = rhs.into();
        Pos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<P: Into<Pos>> std::ops::AddAssign<P> for Pos {
    fn add_assign(&mut self, rhs: P) {
        let rhs = rhs.into();
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<P: Into<Pos>> std::ops::SubAssign<P> for Pos {
    fn sub_assign(&mut self, rhs: P) {
        let rhs = rhs.into();
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

#[inline(always)]
pub const fn pos(x: i8, y: i8) -> Pos {
    Pos { x, y }
}

pub fn add_building(sim: &mut Sim, building: Building) -> crate::Result<()> {
    let id = sim.next_id();

    let res = || -> crate::Result<()> {
        sim.buildings.push(building);
        let building = sim.building(id);
        let pos = building.pos;

        match &building.kind {
            BuildingKind::Deposit(deposit) => {
                let height = deposit.height as i8;
                let width = deposit.width as i8;
                for y in (pos.y)..(pos.y + height) {
                    for x in (pos.x)..(pos.x + width) {
                        place_cell(sim, (x, y), Cell::output(id))?;
                    }
                }

                for x in (pos.x)..(pos.x + width) {
                    check_adjacent_cells(sim, (x, pos.y), (x, pos.y - 1))?;
                }
                for y in (pos.y)..(pos.y + height) {
                    check_adjacent_cells(sim, (pos.x, y), (pos.x - 1, y))?;
                    check_adjacent_cells(sim, (pos.x + width - 1, y), (pos.x + width, y))?;
                }
                for x in (pos.x)..(pos.x + width) {
                    check_adjacent_cells(sim, (x, pos.y + height - 1), (x, pos.y + height))?;
                }
            }
            BuildingKind::Obstacle(obstacle) => {
                let height = obstacle.height;
                let width = obstacle.width;
                for y in 0..height {
                    for x in 0..width {
                        place_cell(sim, pos + (x as i8, y as i8), Cell::inert(id))?;
                    }
                }
            }
            BuildingKind::Mine(mine) => {
                let rot = mine.rotation as usize;
                for (pos, ty) in MINE_CELLS[rot] {
                    place_cell(sim, pos + pos, Cell::new(ty, id))?;
                }
                for (a, b) in ADJACENT_MINE_CELLS[rot] {
                    check_adjacent_cells(sim, pos + a, pos + b)?;
                }
            }
            BuildingKind::Conveyor(conveyor) => {
                let rot = conveyor.rotation as usize;
                if conveyor.big {
                    for (pos, ty) in BIG_CONVEYOR_CELLS[rot] {
                        place_cell(sim, pos + pos, Cell::new(ty, id))?;
                    }
                    for (a, b) in ADJACENT_BIG_CONVEYOR_CELLS[rot] {
                        check_adjacent_cells(sim, pos + a, pos + b)?;
                    }
                } else {
                    for (pos, ty) in SMALL_CONVEYOR_CELLS[rot] {
                        place_cell(sim, pos + pos, Cell::new(ty, id))?;
                    }
                    for (a, b) in ADJACENT_SMALL_CONVEYOR_CELLS[rot] {
                        check_adjacent_cells(sim, pos + a, pos + b)?;
                    }
                }
            }
            BuildingKind::Combiner(combiner) => {
                let rot = combiner.rotation as usize;
                for (pos, ty) in COMBINER_CELLS[rot] {
                    place_cell(sim, pos + pos, Cell::new(ty, id))?;
                }
                for (a, b) in ADJACENT_COMBINER_CELLS[rot] {
                    check_adjacent_cells(sim, pos + a, pos + b)?;
                }
            }
            BuildingKind::Factory(_) => {
                for y in (pos.y)..(pos.y + FACTORY_SIZE) {
                    for x in (pos.x)..(pos.x + FACTORY_SIZE) {
                        place_cell(sim, (x, y), Cell::output(id))?;
                    }
                }
                for x in (pos.x)..(pos.x + FACTORY_SIZE) {
                    check_adjacent_cells(sim, (x, pos.y), (x, pos.y - 1))?;
                }
                for y in (pos.y)..(pos.y + FACTORY_SIZE) {
                    check_adjacent_cells(sim, (pos.x, y), (pos.x - 1, y))?;
                    check_adjacent_cells(
                        sim,
                        (pos.x + FACTORY_SIZE - 1, y),
                        (pos.x + FACTORY_SIZE, y),
                    )?;
                }
                for x in (pos.x)..(pos.x + FACTORY_SIZE) {
                    check_adjacent_cells(
                        sim,
                        (x, pos.y + FACTORY_SIZE - 1),
                        (x, pos.y + FACTORY_SIZE),
                    )?;
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

        sim.connections.retain(|c| c.input != id && c.output != id);
    }

    res
}

fn place_cell(sim: &mut Sim, pos: impl Into<Pos>, cell: Cell) -> crate::Result<()> {
    let pos = pos.into();

    if pos.y < 0 || pos.y >= sim.board.height as i8 {
        return Err(Error::OutOfBounds(pos));
    }
    if pos.x < 0 || pos.x >= sim.board.width as i8 {
        return Err(Error::OutOfBounds(pos));
    }

    if let Some(other) = sim.board[pos] {
        match (&sim.building(other.id).kind, &sim.building(cell.id).kind) {
            (BuildingKind::Conveyor(_), BuildingKind::Conveyor(_))
                if cell.kind != CellKind::Inert || other.kind != CellKind::Inert => {}
            _ => return Err(Error::Interseciton(pos)),
        }
    }

    sim.board[pos] = Some(cell);

    Ok(())
}

fn check_adjacent_cells(
    sim: &mut Sim,
    pos_a: impl Into<Pos>,
    pos_b: impl Into<Pos>,
) -> crate::Result<()> {
    let pos_a = pos_a.into();
    let pos_b = pos_b.into();

    let (a, b) = match (sim.board.get(pos_a), sim.board.get(pos_b)) {
        (Some(a), Some(b)) => (a, b),
        (_, _) => return Ok(()),
    };

    match (a.kind, b.kind) {
        (CellKind::Output, CellKind::Input) => check_connection(sim, pos_a, a, pos_b, b),
        (CellKind::Input, CellKind::Output) => check_connection(sim, pos_b, b, pos_a, a),
        (_, _) => Ok(()),
    }
}

fn check_connection(
    sim: &mut Sim,
    output_pos: Pos,
    output: Cell,
    input_pos: Pos,
    input: Cell,
) -> crate::Result<()> {
    let building_a = sim.building(output.id);
    let building_b = sim.building(input.id);

    match &building_a.kind {
        BuildingKind::Deposit(_) => match &building_b.kind {
            BuildingKind::Deposit(_) => unreachable!(),
            BuildingKind::Obstacle(_) => unreachable!(),
            BuildingKind::Mine(_) => Ok(add_connection(sim, output.id, input.id)),
            BuildingKind::Conveyor(_) | BuildingKind::Combiner(_) | BuildingKind::Factory(_) => {
                Err(Error::DepositEgress(input_pos))
            }
        },
        BuildingKind::Obstacle(_) => unreachable!(),
        BuildingKind::Mine(_) => match &building_b.kind {
            BuildingKind::Deposit(_) => unreachable!(),
            BuildingKind::Obstacle(_) => unreachable!(),
            BuildingKind::Mine(_) => Err(Error::MineEgress(output_pos)),
            BuildingKind::Conveyor(_) | BuildingKind::Combiner(_) | BuildingKind::Factory(_) => {
                Ok(add_connection(sim, output.id, input.id))
            }
        },
        BuildingKind::Conveyor(_) | BuildingKind::Combiner(_) => match &building_b.kind {
            BuildingKind::Deposit(_) => unreachable!(),
            BuildingKind::Obstacle(_) => unreachable!(),
            BuildingKind::Mine(_)
            | BuildingKind::Conveyor(_)
            | BuildingKind::Combiner(_)
            | BuildingKind::Factory(_) => Ok(add_connection(sim, output.id, input.id)),
        },
        BuildingKind::Factory(_) => unreachable!(),
    }
}

fn add_connection(sim: &mut Sim, output: Id, input: Id) {
    for c in sim.connections.iter() {
        if c.output == output {
            if c.input == input {
                break;
            } else {
                todo!("some buildings can't have multiple inputs connected to their output");
            }
        }
    }

    if !sim
        .connections
        .iter()
        .any(|c| c.input == input && c.output == output)
    {
        sim.connections.push(Connection::new(input, output));
    }
}
