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
    pub turns: u32,
    pub time: u32,
}

impl Sim {
    pub fn new(products: Products, board: Board, turns: u32, time: u32) -> Self {
        Self {
            products,
            buildings: Buildings::default(),
            board,
            connections: Vec::new(),
            turns,
            time,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Buildings {
    pub values: Vec<Option<Building>>,
}

impl std::ops::Index<Id> for Buildings {
    type Output = Building;

    fn index(&self, id: Id) -> &Self::Output {
        self.values[id.0 as usize]
            .as_ref()
            .expect("Expected building")
    }
}

impl std::ops::IndexMut<Id> for Buildings {
    fn index_mut(&mut self, id: Id) -> &mut Self::Output {
        self.values[id.0 as usize]
            .as_mut()
            .expect("Expected building")
    }
}

impl Buildings {
    pub fn remove(&mut self, id: Id) -> Building {
        self.values[id.0 as usize]
            .take()
            .expect("Expected building")
    }

    pub fn push(&mut self, value: Building) -> Id {
        match self.values.iter().position(|b| b.is_none()) {
            Some(id) => {
                self.values[id] = Some(value);
                Id(id as u16)
            }
            None => {
                let id = Id(self.values.len() as u16);
                self.values.push(Some(value));
                id
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &Building)> {
        self.values
            .iter()
            .enumerate()
            .filter_map(|(i, b)| b.as_ref().map(|b| (Id(i as u16), b)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut Building)> {
        self.values
            .iter_mut()
            .enumerate()
            .filter_map(|(i, b)| b.as_mut().map(|b| (Id(i as u16), b)))
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deposit {
    pub pos: Pos,
    pub width: u8,
    pub height: u8,
    pub resource_type: ResourceType,
}

impl Deposit {
    pub fn new(pos: impl Into<Pos>, width: u8, height: u8, resource_type: ResourceType) -> Self {
        Self {
            pos: pos.into(),
            width,
            height,
            resource_type,
        }
    }

    pub fn resources(&self) -> u16 {
        5 * self.width as u16 * self.height as u16
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mine {
    pub pos: Pos,
    pub rotation: Rotation,
}

impl Mine {
    pub fn new(pos: impl Into<Pos>, rotation: Rotation) -> Self {
        Self {
            pos: pos.into(),
            rotation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conveyor {
    pub pos: Pos,
    pub rotation: Rotation,
    pub big: bool,
}

impl Conveyor {
    pub fn new(pos: impl Into<Pos>, rotation: Rotation, big: bool) -> Self {
        Self {
            pos: pos.into(),
            rotation,
            big,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combiner {
    pub pos: Pos,
    pub rotation: Rotation,
}

impl Combiner {
    pub fn new(pos: impl Into<Pos>, rotation: Rotation) -> Self {
        Self {
            pos: pos.into(),
            rotation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Factory {
    pub pos: Pos,
    pub product_type: ProductType,
}

impl Factory {
    pub fn new(pos: impl Into<Pos>, product_type: ProductType) -> Self {
        Self {
            pos: pos.into(),
            product_type,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Products {
    values: [Product; 8],
}

impl Default for Products {
    fn default() -> Self {
        Self {
            values: array::from_fn(|_| Product::default()),
        }
    }
}

impl std::ops::Index<ProductType> for Products {
    type Output = Product;

    fn index(&self, index: ProductType) -> &Self::Output {
        &self.values[index as usize]
    }
}

impl std::ops::IndexMut<ProductType> for Products {
    fn index_mut(&mut self, index: ProductType) -> &mut Self::Output {
        &mut self.values[index as usize]
    }
}

impl Products {
    pub fn iter(&self) -> impl Iterator<Item = &Product> {
        self.values.iter()
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

impl std::ops::Index<ResourceType> for Resources {
    type Output = u16;

    fn index(&self, index: ResourceType) -> &Self::Output {
        &self.values[index as usize]
    }
}

impl std::ops::IndexMut<ResourceType> for Resources {
    fn index_mut(&mut self, index: ResourceType) -> &mut Self::Output {
        &mut self.values[index as usize]
    }
}

impl std::ops::AddAssign for Resources {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..RESOURCE_TYPES {
            self.values[i] += rhs.values[i];
        }
    }
}

impl std::ops::SubAssign for Resources {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..RESOURCE_TYPES {
            self.values[i] -= rhs.values[i];
        }
    }
}

impl std::ops::Div for Resources {
    type Output = Resources;

    fn div(self, rhs: Self) -> Resources {
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

    pub fn iter(&self) -> impl Iterator<Item = u16> + '_ {
        self.values.iter().copied()
    }
}
