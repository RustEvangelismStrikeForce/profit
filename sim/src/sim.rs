//! origin: #
//! inert:  o
//! input:  +
//! output: -

use std::array;

use crate::{Board, Connection, Id, Pos, Rotation};

pub const RESOURCE_TYPES: usize = 8;
pub const PRODUCT_TYPES: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sim {
    pub products: Products,
    pub buildings: Buildings,
    pub board: Board,
    pub connections: Vec<Connection>,
}

impl Sim {
    pub fn new(products: Products, board: Board) -> Self {
        Self {
            products,
            buildings: Buildings::default(),
            board,
            connections: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Buildings {
    pub values: Vec<Building>,
}

impl std::ops::Index<Id> for Buildings {
    type Output = Building;

    fn index(&self, id: Id) -> &Self::Output {
        &self.values[id.0 as usize]
    }
}

impl std::ops::IndexMut<Id> for Buildings {
    fn index_mut(&mut self, id: Id) -> &mut Self::Output {
        &mut self.values[id.0 as usize]
    }
}

impl Buildings {
    pub fn next_id(&self) -> Id {
        Id(self.values.len() as u16)
    }

    pub fn push(&mut self, value: Building) {
        self.values.push(value)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Building> {
        self.values.iter_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Building> {
        self.values.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Building {
    Deposit(Deposit),
    Obstacle(Obstacle),
    Mine(Mine),
    Conveyor(Conveyor),
    Combiner(Combiner),
    Factory(Factory),
}

impl Building {
    pub fn pos(&self) -> Pos {
        match self {
            Building::Deposit(d) => d.pos,
            Building::Obstacle(o) => o.pos,
            Building::Mine(m) => m.pos,
            Building::Conveyor(c) => c.pos,
            Building::Combiner(c) => c.pos,
            Building::Factory(f) => f.pos,
        }
    }

    pub fn output_resources(&mut self) -> Resources {
        match self {
            Building::Deposit(deposit) => {
                let num = deposit.resources.min(3);
                deposit.resources -= num;

                let mut res = Resources::default();
                res.values[deposit.resource_type as usize] += num;
                res
            }
            Building::Obstacle(_) => unreachable!("Obstacles cannot contain resources"),
            Building::Mine(mine) => std::mem::take(&mut mine.resources),
            Building::Conveyor(conveyor) => std::mem::take(&mut conveyor.resources),
            Building::Combiner(combiner) => std::mem::take(&mut combiner.resources),
            Building::Factory(_) => unreachable!("Facotories cannot output resources"),
        }
    }

    pub fn input_resources(&mut self, res: Resources) {
        match self {
            Building::Deposit(_) => unreachable!("Deposits cannot input resources"),
            Building::Obstacle(_) => unreachable!("Obstacles cannot contain resources"),
            Building::Mine(mine) => mine.resources += res,
            Building::Conveyor(conveyor) => conveyor.resources += res,
            Building::Combiner(combiner) => combiner.resources += res,
            Building::Factory(factory) => factory.resources += res,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deposit {
    pub pos: Pos,
    pub width: u8,
    pub height: u8,
    pub resource_type: ResourceType,
    pub resources: u16,
}

impl Deposit {
    pub fn new(pos: impl Into<Pos>, width: u8, height: u8, resource_type: ResourceType) -> Self {
        Self {
            pos: pos.into(),
            width,
            height,
            resource_type,
            resources: width as u16 * height as u16 * 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Obstacle {
    pub pos: Pos,
    pub width: u8,
    pub height: u8,
}

impl Obstacle {
    pub fn new(pos: impl Into<Pos>, width: u8, height: u8) -> Self {
        Self {
            pos: pos.into(),
            width,
            height,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mine {
    pub pos: Pos,
    pub rotation: Rotation,
    pub resources: Resources,
}

impl Mine {
    pub fn new(pos: impl Into<Pos>, rotation: Rotation) -> Self {
        Self {
            pos: pos.into(),
            rotation,
            resources: Resources::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conveyor {
    pub pos: Pos,
    pub rotation: Rotation,
    pub big: bool,
    pub resources: Resources,
}

impl Conveyor {
    pub fn new(pos: impl Into<Pos>, rotation: Rotation, big: bool) -> Self {
        Self {
            pos: pos.into(),
            rotation,
            big,
            resources: Resources::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combiner {
    pub pos: Pos,
    pub rotation: Rotation,
    pub resources: Resources,
}

impl Combiner {
    pub fn new(pos: impl Into<Pos>, rotation: Rotation) -> Self {
        Self {
            pos: pos.into(),
            rotation,
            resources: Resources::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Factory {
    pub pos: Pos,
    pub product_type: ProductType,
    pub resources: Resources,
}

impl Factory {
    pub fn new(pos: impl Into<Pos>, product_type: ProductType) -> Self {
        Self {
            pos: pos.into(),
            product_type,
            resources: Resources::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Products {
    values: [Product; 8],
}

impl std::ops::Deref for Products {
    type Target = [Product; 8];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl std::ops::DerefMut for Products {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl Default for Products {
    fn default() -> Self {
        Self {
            values: array::from_fn(|_| Product::default()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceType {
    Type0 = 0,
    Type1 = 1,
    Type2 = 2,
    Type3 = 3,
    Type4 = 4,
    Type5 = 5,
    Type6 = 6,
    Type7 = 7,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Resources {
    pub values: [u16; RESOURCE_TYPES],
}

impl std::ops::AddAssign for Resources {
    fn add_assign(&mut self, rhs: Self) {
        // TODO: simd
        for i in 0..RESOURCE_TYPES {
            self.values[i] += rhs.values[i];
        }
    }
}

impl std::ops::SubAssign for Resources {
    fn sub_assign(&mut self, rhs: Self) {
        // TODO: simd
        for i in 0..RESOURCE_TYPES {
            self.values[i] -= rhs.values[i];
        }
    }
}

impl std::ops::Div for Resources {
    type Output = Resources;

    fn div(self, rhs: Self) -> Resources {
        // TODO: simd
        let mut res = Resources::default();
        for i in 0..RESOURCE_TYPES {
            res.values[i] = self.values[i]
                .checked_div(rhs.values[i])
                .unwrap_or(u16::MAX);
        }
        res
    }
}

impl std::ops::Mul for Resources {
    type Output = Resources;

    fn mul(self, rhs: Self) -> Resources {
        // TODO: simd
        let mut res = Resources::default();
        for i in 0..RESOURCE_TYPES {
            res.values[i] = self.values[i] * rhs.values[i];
        }
        res
    }
}

impl Resources {
    pub fn new(values: [u16; RESOURCE_TYPES]) -> Self {
        Self { values }
    }

    pub fn has_at_least(&self, other: &Resources) -> bool {
        for i in 0..RESOURCE_TYPES {
            if self.values[i] < other.values[i] {
                return false;
            }
        }
        true
    }

    pub fn is_empty(&self) -> bool {
        self.values.iter().all(|r| *r == 0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimRun {
    pub rounds: u32,
    pub points: u32,
    pub at_turn: u32,
}

pub fn run(sim: &mut Sim, max_rounds: u32) -> SimRun {
    let mut points = 0;
    let mut rounds = 0;
    let mut at_turn = 0;

    while rounds < max_rounds {
        let mut unchanged = true;

        // start of the round
        for con in sim.connections.iter_mut() {
            let building_b = &mut sim.buildings[con.input_id];
            let res = std::mem::take(&mut con.resources);
            unchanged &= res.is_empty();
            building_b.input_resources(res);
        }

        for con in sim.connections.iter_mut() {
            let building_a = &mut sim.buildings[con.output_id];
            con.resources = building_a.output_resources();
            unchanged &= con.resources.is_empty();
        }

        for b in sim.buildings.iter_mut() {
            let Building::Factory(f) = b else { continue };
            let product = &sim.products.values[f.product_type as usize];
            if f.resources.has_at_least(&product.resources) {
                let times = (f.resources / product.resources)
                    .values
                    .iter()
                    .min()
                    .map_or(0, |t| *t);

                if times > 0 {
                    f.resources -= product.resources * Resources::new([times; 8]);
                    points += product.points * times as u32;
                    at_turn = rounds + 1;
                    unchanged = false;
                }
            }
        }

        if unchanged {
            break;
        }

        rounds += 1;
    }

    SimRun {
        rounds,
        points,
        at_turn,
    }
}
