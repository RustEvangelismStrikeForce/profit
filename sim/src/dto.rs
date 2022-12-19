use serde::{Deserialize, Serialize};

use crate::{
    place_building, pos, Board, Building, Combiner, Conveyor, Deposit, Error, Factory, IoError,
    Mine, Obstacle, ProductType, Products, ResourceType, Resources, Rotation, Sim,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub width: i8,
    pub height: i8,
    pub objects: Vec<Object>,
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
            if p.subtype >= 8 {
                return Err(Error::Io(IoError::UnknownProductSubtype(p.subtype)));
            }
            sim.products[p.subtype as usize] =
                crate::Product::new(Resources::new(p.resources), p.points);
        }

        for o in task.objects.iter() {
            place_building(&mut sim, o.try_into()?)?;
        }

        Ok(sim)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Solution {
    pub objects: Vec<Object>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Object {
    #[serde(rename = "type")]
    pub kind: ObjectKind,
    #[serde(default)]
    pub subtype: u8,
    pub x: i8,
    pub y: i8,
    // TODO: consider Option<NonZeroU8>
    #[serde(default)]
    pub width: u8,
    #[serde(default)]
    pub height: u8,
}

impl TryFrom<&Object> for Building {
    type Error = IoError;

    fn try_from(o: &Object) -> Result<Self, IoError> {
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
                    0 => Rotation::Up,
                    1 => Rotation::Right,
                    2 => Rotation::Down,
                    3 => Rotation::Left,
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
                        0 => Rotation::Up,
                        1 => Rotation::Right,
                        2 => Rotation::Down,
                        3 => Rotation::Left,
                        _ => unreachable!(),
                    },
                    o.subtype / 4 == 1,
                ))
            }
            ObjectKind::Combiner => Building::Combiner(Combiner::new(
                pos,
                match o.subtype {
                    0 => Rotation::Up,
                    1 => Rotation::Right,
                    2 => Rotation::Down,
                    3 => Rotation::Left,
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
