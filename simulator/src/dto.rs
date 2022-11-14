use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub width: u8,
    pub height: u8,
    pub objects: Vec<Object>,
    pub products: Vec<Product>,
    pub turns: usize,
    pub time: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Object {
    #[serde(rename = "type")]
    pub kind: ObjectKind,
    #[serde(default)]
    pub subtype: u8,
    pub x: usize,
    pub y: usize,
    pub width: Option<usize>,
    pub height: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    Deposit,
    Obstacle,
    Conveyor,
    Combiner,
    Mine,
    Factory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Product {
    pub subtype: u8,
    pub resources: [u16; 8],
    pub points: usize,
}
