use serde::{Deserialize, Serialize};

use crate::{
    place_building, pos, Board, Building, Combiner, Conveyor, Deposit, Error, Factory, IoError,
    Mine, Obstacle, ProductType, Products, ResourceType, Resources, Rotation, Sim,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub width: i8,
    pub height: i8,
    pub objects: Vec<TaskObject>,
    pub products: Vec<Product>,
    pub turns: u32,
    pub time: u32,
}

impl TryFrom<&Task> for Sim {
    type Error = Error;

    fn try_from(task: &Task) -> Result<Self, Self::Error> {
        let products = Products::default();
        let board = Board::new(task.width, task.height);
        let mut sim = Sim::new(products, board, task.turns, task.time);

        for p in task.products.iter() {
            let product_type = ProductType::try_from(p.subtype)?;
            sim.products[product_type] = crate::Product::new(Resources::new(p.resources), p.points);
        }

        for o in task.objects.iter() {
            place_building(&mut sim, o.try_into()?)?;
        }

        Ok(sim)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskObject {
    #[serde(rename = "type")]
    pub kind: ObjectKind,
    #[serde(default)]
    pub subtype: u8,
    pub x: i8,
    pub y: i8,
    #[serde(default)]
    pub width: u8,
    #[serde(default)]
    pub height: u8,
}

impl TryFrom<&TaskObject> for Building {
    type Error = IoError;

    fn try_from(o: &TaskObject) -> Result<Self, IoError> {
        let pos = pos(o.x, o.y);
        Ok(match o.kind {
            ObjectKind::Deposit => Building::Deposit(Deposit::new(
                pos,
                o.width,
                o.height,
                match o.subtype {
                    0 => ResourceType::Type0,
                    1 => ResourceType::Type1,
                    2 => ResourceType::Type2,
                    3 => ResourceType::Type3,
                    4 => ResourceType::Type4,
                    5 => ResourceType::Type5,
                    6 => ResourceType::Type6,
                    7 => ResourceType::Type7,
                    t => return Err(IoError::UnknownDepositSubtype(t)),
                },
            )),
            ObjectKind::Obstacle => Building::Obstacle(Obstacle::new(pos, o.width, o.height)),
            ObjectKind::Mine => Building::Mine(Mine::new(
                pos,
                match o.subtype {
                    0 => Rotation::Right,
                    1 => Rotation::Down,
                    2 => Rotation::Left,
                    3 => Rotation::Up,
                    t => return Err(IoError::UnknownMineSubtype(t)),
                },
            )),
            ObjectKind::Conveyor => {
                if o.subtype >= 8 {
                    return Err(IoError::UnknownMineSubtype(o.subtype));
                }
                Building::Conveyor(Conveyor::new(
                    pos,
                    match o.subtype % 4 {
                        0 => Rotation::Right,
                        1 => Rotation::Down,
                        2 => Rotation::Left,
                        3 => Rotation::Up,
                        _ => unreachable!(),
                    },
                    o.subtype / 4 == 1,
                ))
            }
            ObjectKind::Combiner => Building::Combiner(Combiner::new(
                pos,
                match o.subtype {
                    0 => Rotation::Right,
                    1 => Rotation::Down,
                    2 => Rotation::Left,
                    3 => Rotation::Up,
                    t => return Err(IoError::UnknownCombinerSubtype(t)),
                },
            )),
            ObjectKind::Factory => Building::Factory(Factory::new(
                pos,
                match o.subtype {
                    0 => ProductType::Type0,
                    1 => ProductType::Type1,
                    2 => ProductType::Type2,
                    3 => ProductType::Type3,
                    4 => ProductType::Type4,
                    5 => ProductType::Type5,
                    6 => ProductType::Type6,
                    7 => ProductType::Type7,
                    t => return Err(IoError::UnknownFactorySubtype(t)),
                },
            )),
        })
    }
}

impl TryFrom<u8> for ProductType {
    type Error = IoError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ProductType::Type0),
            1 => Ok(ProductType::Type1),
            2 => Ok(ProductType::Type2),
            3 => Ok(ProductType::Type3),
            4 => Ok(ProductType::Type4),
            5 => Ok(ProductType::Type5),
            6 => Ok(ProductType::Type6),
            7 => Ok(ProductType::Type7),
            t => Err(IoError::UnknownProductSubtype(t)),
        }
    }
}

impl TryFrom<u8> for ResourceType {
    type Error = IoError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ResourceType::Type0),
            1 => Ok(ResourceType::Type1),
            2 => Ok(ResourceType::Type2),
            3 => Ok(ResourceType::Type3),
            4 => Ok(ResourceType::Type4),
            5 => Ok(ResourceType::Type5),
            6 => Ok(ResourceType::Type6),
            7 => Ok(ResourceType::Type7),
            t => Err(IoError::UnknownResourceType(t)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Solution {
    pub objects: Vec<SolutionObject>,
}

impl From<&Sim> for Solution {
    fn from(sim: &Sim) -> Self {
        let objects = sim
            .buildings
            .iter()
            .filter_map(|(_, b)| SolutionObject::try_from(b).ok())
            .collect();
        Solution { objects }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolutionObject {
    #[serde(rename = "type")]
    pub kind: ObjectKind,
    #[serde(default)]
    pub subtype: u8,
    pub x: i8,
    pub y: i8,
}

impl TryFrom<&Building> for SolutionObject {
    type Error = ();

    fn try_from(building: &Building) -> Result<SolutionObject, ()> {
        match building {
            Building::Deposit(_) => Err(()),
            Building::Obstacle(_) => Err(()),
            Building::Mine(mine) => Ok(SolutionObject {
                kind: ObjectKind::Mine,
                subtype: mine.rotation as u8,
                x: mine.pos.x,
                y: mine.pos.y,
            }),
            Building::Conveyor(conveyor) => Ok(SolutionObject {
                kind: ObjectKind::Conveyor,
                subtype: conveyor.rotation as u8 + 4 * (conveyor.big as u8),
                x: conveyor.pos.x,
                y: conveyor.pos.y,
            }),
            Building::Combiner(combiner) => Ok(SolutionObject {
                kind: ObjectKind::Combiner,
                subtype: combiner.rotation as u8,
                x: combiner.pos.x,
                y: combiner.pos.y,
            }),
            Building::Factory(factory) => Ok(SolutionObject {
                kind: ObjectKind::Factory,
                subtype: factory.product_type as u8,
                x: factory.pos.x,
                y: factory.pos.y,
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    Deposit,
    Obstacle,
    Mine,
    Conveyor,
    Combiner,
    Factory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Product {
    pub subtype: u8,
    pub resources: [u16; 8],
    pub points: u32,
}
