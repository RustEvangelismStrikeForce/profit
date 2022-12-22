use core::fmt;

use crate::{Building, Error, Resources, Sim};

pub const MAX_BOARD_SIZE: i8 = 100;
pub const FACTORY_SIZE: i8 = 5;

const MINE_CELLS: [[(Pos, CellKind); 6]; 4] = [
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
const ADJACENT_MINE_CELLS: [[(Pos, Pos); 6]; 4] = [
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

const SMALL_CONVEYOR_CELLS: [[(Pos, CellKind); 3]; 4] = [
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
const ADJACENT_SMALL_CONVEYOR_CELLS: [[(Pos, Pos); 6]; 4] = [
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

const BIG_CONVEYOR_CELLS: [[(Pos, CellKind); 4]; 4] = [
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
const ADJACENT_BIG_CONVEYOR_CELLS: [[(Pos, Pos); 6]; 4] = [
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
const COMBINER_CELLS: [[(Pos, CellKind); 7]; 4] = [
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
const ADJACENT_COMBINER_CELLS: [[(Pos, Pos); 8]; 4] = [
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

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    pub width: i8,
    pub height: i8,
    cells: Vec<Option<Cell>>,
}

impl<P: Into<Pos>> std::ops::Index<P> for Board {
    type Output = Option<Cell>;

    fn index(&self, pos: P) -> &Self::Output {
        let pos = pos.into();
        assert!(
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height,
            "Board index out of bounds: {pos}"
        );

        &self.cells[pos.y as usize * self.width as usize + pos.x as usize]
    }
}

impl<P: Into<Pos>> std::ops::IndexMut<P> for Board {
    fn index_mut(&mut self, pos: P) -> &mut Self::Output {
        let pos = pos.into();
        assert!(
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height,
            "Board index out of bounds: {pos}"
        );

        &mut self.cells[pos.y as usize * self.width as usize + pos.x as usize]
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\n\x1B[7;94m    \x1B[0m")?;
        for x in 0..self.width {
            write!(f, "\x1B[7;94m{x:3}\x1B[0m")?;
        }
        for y in 0..self.height {
            write!(f, "\n\x1B[1;7;94m{y:3}\x1B[0m ")?;
            for x in 0..self.width {
                match self[pos(x, y)] {
                    Some(c) => match c.kind {
                        CellKind::Input => write!(f, "  i")?,
                        CellKind::Output => write!(f, "  o")?,
                        CellKind::Inert => write!(f, "  x")?,
                    },
                    None => write!(f, "  .",)?,
                }
            }
        }

        Ok(())
    }
}

impl Board {
    pub fn new(width: i8, height: i8) -> Self {
        let width = width.clamp(0, MAX_BOARD_SIZE);
        let height = height.clamp(0, MAX_BOARD_SIZE);
        Board {
            width,
            height,
            cells: vec![None; width as usize * height as usize],
        }
    }

    pub fn get(&self, pos: impl Into<Pos>) -> Option<Option<Cell>> {
        let pos = pos.into();
        if pos.x < 0 || pos.x >= self.width {
            return None;
        }
        if pos.y < 0 || pos.y >= self.height {
            return None;
        }
        Some(self[pos])
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub kind: CellKind,
    pub id: Id,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u16);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CellKind {
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

impl Rotation {
    pub fn is_vertical(&self) -> bool {
        *self as u8 ^ 1 == 0
    }

    pub fn is_horizontal(&self) -> bool {
        *self as u8 ^ 1 == 1
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pos {
    pub x: i8,
    pub y: i8,
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:2}, {:2})", self.x, self.y)
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

impl Pos {
    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }

    pub fn manhattan_len(&self) -> u8 {
        self.x.unsigned_abs() + self.y.unsigned_abs()
    }

    pub fn rot_90(&self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }
}

#[inline(always)]
pub const fn pos(x: i8, y: i8) -> Pos {
    Pos { x, y }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    /// Output cell - input of the connection
    pub output_id: Id,
    pub output_pos: Pos,
    /// Input cell - output of the connection
    pub input_id: Id,
    pub input_pos: Pos,
    pub resources: Resources,
}

impl Connection {
    pub fn new(output_id: Id, output_pos: Pos, input_id: Id, input_pos: Pos) -> Self {
        Self {
            output_id,
            output_pos,
            input_id,
            input_pos,
            resources: Resources::default(),
        }
    }
}

pub fn remove_building(sim: &mut Sim, id: Id) -> Building {
    let building = sim.buildings.remove(id);
    let (is_conveyor, is_vertical) = match &building {
        Building::Conveyor(c) => (true, c.rotation.is_vertical()),
        _ => (false, false),
    };

    for y in 0..sim.board.height {
        for x in 0..sim.board.width {
            let pos = pos(x, y);
            if let Some(c) = sim.board[pos] {
                if c.id == id {
                    sim.board[pos] = None;

                    // check for conveyor intersections
                    if is_conveyor && c.kind == CellKind::Inert {
                        if is_vertical {
                            let Some(left) =  sim.board.get(pos + (-1, 0)).flatten() else { continue };
                            let Some(right) =  sim.board.get(pos + (1, 0)).flatten() else { continue };

                            let mut intersecting_id = left.id;
                            if left.id != right.id {
                                let mut matches = false;
                                if let Some(two_left) = sim.board.get(pos + (-2, 0)).flatten() {
                                    if two_left.id == right.id {
                                        intersecting_id = right.id;
                                        matches = true;
                                    }
                                }
                                if let Some(two_right) = sim.board.get(pos + (2, 0)).flatten() {
                                    if two_right.id == left.id {
                                        intersecting_id = left.id;
                                        matches = true;
                                    }
                                }
                                if !matches {
                                    continue;
                                }
                            }

                            sim.board[pos] = Some(Cell::inert(intersecting_id));
                        } else {
                            let Some(up) =  sim.board.get(pos + (0, -1)).flatten() else { continue };
                            let Some(down) =  sim.board.get(pos + (0, 1)).flatten() else { continue };

                            let mut intersecting_id = up.id;
                            if up.id != down.id {
                                let mut matches = false;
                                if let Some(two_up) = sim.board.get(pos + (0, -2)).flatten() {
                                    if two_up.id == down.id {
                                        intersecting_id = down.id;
                                        matches = true;
                                    }
                                }
                                if let Some(two_down) = sim.board.get(pos + (0, 2)).flatten() {
                                    if two_down.id == up.id {
                                        intersecting_id = up.id;
                                        matches = true;
                                    }
                                }
                                if !matches {
                                    continue;
                                }
                            }

                            sim.board[pos] = Some(Cell::inert(intersecting_id));
                        }
                    }
                }
            }
        }
    }

    sim.connections
        .retain(|c| c.input_id != id && c.output_id != id);

    building
}

pub fn place_building(sim: &mut Sim, building: Building) -> crate::Result<Id> {
    let id = sim.buildings.push(building);

    // TODO: try blocks https://doc.rust-lang.org/beta/unstable-book/language-features/try-blocks.html
    let res = || -> crate::Result<()> {
        let building = &sim.buildings[id];

        match &building {
            Building::Deposit(deposit) => {
                let pos = deposit.pos;
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
            Building::Obstacle(obstacle) => {
                let pos = obstacle.pos;
                let height = obstacle.height;
                let width = obstacle.width;
                for y in 0..height {
                    for x in 0..width {
                        place_cell(sim, pos + (x as i8, y as i8), Cell::inert(id))?;
                    }
                }
            }
            Building::Mine(mine) => {
                let pos = mine.pos;
                let rot = mine.rotation as usize;
                for (p, ty) in MINE_CELLS[rot] {
                    place_cell(sim, pos + p, Cell::new(ty, id))?;
                }
                for (a, b) in ADJACENT_MINE_CELLS[rot] {
                    check_adjacent_cells(sim, pos + a, pos + b)?;
                }
            }
            Building::Conveyor(conveyor) => {
                let pos = conveyor.pos;
                let rot = conveyor.rotation as usize;
                if conveyor.big {
                    for (p, ty) in BIG_CONVEYOR_CELLS[rot] {
                        place_cell(sim, pos + p, Cell::new(ty, id))?;
                    }
                    for (a, b) in ADJACENT_BIG_CONVEYOR_CELLS[rot] {
                        check_adjacent_cells(sim, pos + a, pos + b)?;
                    }
                } else {
                    for (p, ty) in SMALL_CONVEYOR_CELLS[rot] {
                        place_cell(sim, pos + p, Cell::new(ty, id))?;
                    }
                    for (a, b) in ADJACENT_SMALL_CONVEYOR_CELLS[rot] {
                        check_adjacent_cells(sim, pos + a, pos + b)?;
                    }
                }
            }
            Building::Combiner(combiner) => {
                let pos = combiner.pos;
                let rot = combiner.rotation as usize;
                for (p, ty) in COMBINER_CELLS[rot] {
                    place_cell(sim, pos + p, Cell::new(ty, id))?;
                }
                for (a, b) in ADJACENT_COMBINER_CELLS[rot] {
                    check_adjacent_cells(sim, pos + a, pos + b)?;
                }
            }
            Building::Factory(factory) => {
                let pos = factory.pos;
                for y in (pos.y)..(pos.y + FACTORY_SIZE) {
                    for x in (pos.x)..(pos.x + FACTORY_SIZE) {
                        place_cell(sim, (x, y), Cell::input(id))?;
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
        sim.buildings.values[id.0 as usize] = None;

        for c in sim.board.cells.iter_mut() {
            if let Some(cell) = c {
                if cell.id == id {
                    *c = None;
                }
            }
        }

        sim.connections
            .retain(|c| c.input_id != id && c.output_id != id);
    }

    res.and(Ok(id))
}

fn place_cell(sim: &mut Sim, pos: impl Into<Pos>, cell: Cell) -> crate::Result<()> {
    let pos = pos.into();

    let Some(other) = sim.board.get(pos) else { return Err(Error::OutOfBounds(pos)) };

    if let Some(other) = other {
        match (&sim.buildings[other.id], &sim.buildings[cell.id]) {
            (Building::Conveyor(_), Building::Conveyor(_))
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
        (Some(Some(a)), Some(Some(b))) => (a, b),
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
    let building_a = &sim.buildings[output.id];
    let building_b = &sim.buildings[input.id];

    match &building_a {
        Building::Deposit(_) => match &building_b {
            Building::Deposit(_) => unreachable!(),
            Building::Obstacle(_) => unreachable!(),
            Building::Mine(_) => {
                let con = Connection::new(output.id, output_pos, input.id, input_pos);

                for c in sim.connections.iter() {
                    if c.output_pos == output_pos {
                        return Err(Error::MultipleIngresses(output_pos));
                    }
                }

                sim.connections.push(con);
                Ok(())
            }
            Building::Conveyor(_) | Building::Combiner(_) | Building::Factory(_) => {
                Err(Error::DepositEgress(input_pos))
            }
        },
        Building::Obstacle(_) => unreachable!(),
        Building::Mine(_) => match &building_b {
            Building::Deposit(_) => unreachable!(),
            Building::Obstacle(_) => unreachable!(),
            Building::Mine(_) => Err(Error::MineEgress(output_pos)),
            Building::Conveyor(_) | Building::Combiner(_) | Building::Factory(_) => {
                let con = Connection::new(output.id, output_pos, input.id, input_pos);

                for c in sim.connections.iter() {
                    if c.output_pos == output_pos {
                        return Err(Error::MultipleIngresses(output_pos));
                    }
                }

                sim.connections.push(con);
                Ok(())
            }
        },
        Building::Conveyor(_) | Building::Combiner(_) => match &building_b {
            Building::Deposit(_) => unreachable!(),
            Building::Obstacle(_) => unreachable!(),
            Building::Mine(_)
            | Building::Conveyor(_)
            | Building::Combiner(_)
            | Building::Factory(_) => {
                let con = Connection::new(output.id, output_pos, input.id, input_pos);

                for c in sim.connections.iter() {
                    if c.output_pos == output_pos {
                        return Err(Error::MultipleIngresses(output_pos));
                    }
                }

                sim.connections.push(con);
                Ok(())
            }
        },
        Building::Factory(_) => unreachable!(),
    }
}
